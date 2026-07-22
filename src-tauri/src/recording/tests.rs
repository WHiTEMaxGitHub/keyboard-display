use super::{
    binary::{decode_kbdrec, encode_kbdrec},
    browser::{create_recording_folder, list_recording_files},
    filename::{format_recording_file_name, parse_recording_file_times},
    inspection::inspect_kbdrec,
    manager::RecordingManager,
    metadata::{read_recording_metadata, save_recording_metadata},
    session::{sample_frames, RecordingSession},
    types::{RecordingEvent, RecordingMarkerNote, RecordingMetadata, RecordingSnapshot},
};

#[test]
fn stores_key_events_with_monotonic_relative_timestamps() {
    let mut session = RecordingSession::new(60, 1000);

    session.record_input(1120, "w", true);
    session.record_input(1190, "w", false);

    assert_eq!(
        session.snapshot().events,
        vec![
            RecordingEvent::KeyDown {
                frame: 7,
                key_id: "w".to_string(),
            },
            RecordingEvent::KeyUp {
                frame: 11,
                key_id: "w".to_string(),
            },
        ],
    );
}

#[test]
fn ignores_duplicate_state_events() {
    let mut session = RecordingSession::new(60, 1000);

    session.record_input(1100, "tab", true);
    session.record_input(1110, "tab", true);
    session.record_input(1120, "tab", false);
    session.record_input(1130, "tab", false);

    assert_eq!(
        session.snapshot().events,
        vec![
            RecordingEvent::KeyDown {
                frame: 6,
                key_id: "tab".to_string(),
            },
            RecordingEvent::KeyUp {
                frame: 7,
                key_id: "tab".to_string(),
            },
        ],
    );
}

#[test]
fn suppresses_only_recent_control_chord_events() {
    let mut session = RecordingSession::new(60, 1000);

    session.record_input(1010, "ctrl-left", true);
    session.record_input(1020, "ctrl-left", false);
    session.record_input(1100, "ctrl-left", true);
    session.record_input(1110, "shift-left", true);
    session.record_input(1120, "r", true);
    session.suppress_recent_keys(&[
        "ctrl-left".to_string(),
        "shift-left".to_string(),
        "r".to_string(),
    ]);

    assert_eq!(
        session.snapshot().events,
        vec![
            RecordingEvent::KeyDown {
                frame: 1,
                key_id: "ctrl-left".to_string(),
            },
            RecordingEvent::KeyUp {
                frame: 1,
                key_id: "ctrl-left".to_string(),
            },
        ],
    );
}

#[test]
fn keeps_single_control_key_when_no_hotkey_is_suppressed() {
    let mut session = RecordingSession::new(60, 1000);

    session.record_input(1010, "ctrl-left", true);
    session.record_input(1020, "ctrl-left", false);

    assert_eq!(session.snapshot().events.len(), 2);
}

#[test]
fn stores_sync_markers_in_event_stream() {
    let mut session = RecordingSession::new(60, 2000);

    session.add_marker(2500, "sync");

    assert_eq!(
        session.snapshot().events,
        vec![RecordingEvent::Marker {
            frame: 30,
            name: "sync".to_string(),
        }],
    );

    let serialized = serde_json::to_value(session.snapshot()).unwrap();
    assert_eq!(serialized["events"][0]["marker"], "sync");
    assert!(serialized["events"][0].get("type").is_none());
}

#[test]
fn manager_adds_markers_to_active_session() {
    let manager = RecordingManager::new();

    manager.start(60, 1000, 1000).unwrap();
    manager.add_marker(1250, "hotkey-start").unwrap();

    let session = manager.session.lock().unwrap();
    let active_session = session.as_ref().unwrap();
    assert_eq!(
        active_session.session.snapshot().events,
        vec![RecordingEvent::Marker {
            frame: 15,
            name: "hotkey-start".to_string(),
        }],
    );
}

#[test]
fn serializes_versioned_recording_envelope() {
    let mut session = RecordingSession::new(60, 3000);

    session.record_input(3016, "space", true);

    let serialized = serde_json::to_value(session.snapshot()).unwrap();

    assert_eq!(serialized["version"], 1);
    assert_eq!(serialized["fps"], 60);
    assert_eq!(serialized["timebase"], "monotonic");
    assert_eq!(serialized["events"][0]["frame"], 1);
    assert_eq!(serialized["events"][0]["down"], "space");
    assert!(serialized["events"][0].get("type").is_none());
    assert!(serialized["events"][0].get("pressed").is_none());
}

#[test]
fn samples_held_keys_by_frame() {
    let frames = sample_frames(
        10,
        300,
        &[
            RecordingEvent::KeyDown {
                frame: 1,
                key_id: "w".to_string(),
            },
            RecordingEvent::KeyUp {
                frame: 3,
                key_id: "w".to_string(),
            },
        ],
    );

    assert_eq!(frames[0].frame, 0);
    assert!(frames[0].keys.is_empty());
    assert_eq!(frames[1].keys, vec!["w".to_string()]);
    assert_eq!(frames[2].keys, vec!["w".to_string()]);
    assert!(frames[3].keys.is_empty());
}

#[test]
fn samples_frames_without_integer_millisecond_drift() {
    let frames = sample_frames(60, 6, &[]);

    assert_eq!(frames[0].frame, 0);
    assert_eq!(frames[3].frame, 3);
    assert_eq!(frames[6].frame, 6);
}

#[test]
fn manager_writes_binary_kbdrec_file() {
    let output_dir = std::env::temp_dir().join(format!(
        "keyboard-display-recording-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_dir);
    let manager = RecordingManager::new();

    manager.start(60, 1000, 1000).unwrap();
    manager.record_input(1016, "w", true).unwrap();
    let result = manager.stop(output_dir.clone(), 1200).unwrap();
    let contents = std::fs::read(&result.path).unwrap();
    let decoded = decode_kbdrec(&contents).unwrap();

    assert_eq!(&contents[0..4], b"KBDR");
    assert_eq!(decoded.frames.len(), 2);
    assert!(decoded.frames[0].keys.is_empty());
    assert_eq!(decoded.frames[1].keys, vec!["w"]);

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn formats_recording_file_name_from_template() {
    assert_eq!(
        format_recording_file_name("${profileName}-${fps}-${start}", 1000, 2000, "Aim", 120),
        "Aim-120-1000.kbdrec",
    );
    assert_eq!(
        format_recording_file_name("../${profileName}\n${end}", 1000, 2000, "CS/POV", 60),
        "..-CS-POV-2000.kbdrec",
    );
    let readable_name = format_recording_file_name(
        "${profileSlug}-${startDate}-${startTime}-${endTime}-${duration}-${fps}fps",
        1000,
        41_000,
        "CS POV / Aim",
        120,
    );
    assert!(readable_name.starts_with("cs-pov-aim-"));
    assert!(readable_name.ends_with("-00-00-40-120fps.kbdrec"));
    assert!(!readable_name.contains("${"));
}

#[test]
fn manager_uses_filename_template_when_writing_recording() {
    let output_dir = std::env::temp_dir().join(format!(
        "keyboard-display-recording-template-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_dir);
    let manager = RecordingManager::new();

    manager.start(60, 1000, 1000).unwrap();
    manager.record_input(1016, "w", true).unwrap();
    let result = manager
        .stop_with_filename_template(
            output_dir.clone(),
            1200,
            "${profileName}-${fps}-${start}",
            "Aim",
            60,
        )
        .unwrap();

    assert!(result.path.ends_with("Aim-60-1000.kbdrec"));

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn binary_recording_roundtrips_frame_state_stream() {
    let snapshot = {
        let mut session = RecordingSession::new(60, 1000);
        session.record_input(1016, "w", true);
        session.record_input(1016, "shift-left", true);
        session.record_input(1083, "shift-left", false);
        session.add_marker(1200, "sync");
        session.snapshot()
    };

    let encoded = encode_kbdrec(&snapshot).unwrap();
    let decoded = decode_kbdrec(&encoded).unwrap();

    assert_eq!(decoded.fps, 60);
    assert_eq!(decoded.key_ids, vec!["w", "shift-left"]);
    assert_eq!(decoded.frames.len(), 13);
    assert!(decoded.frames[0].keys.is_empty());
    assert_eq!(decoded.frames[1].keys, vec!["w", "shift-left"]);
    assert_eq!(decoded.frames[6].keys, vec!["w"]);
    assert_eq!(decoded.markers[0].frame, 12);
    assert_eq!(decoded.markers[0].name, "sync");
    assert!(decoded.runs.len() < decoded.frames.len());
}

#[test]
fn binary_recording_deduplicates_markers_by_frame_and_name() {
    let snapshot = RecordingSnapshot {
        version: 1,
        fps: 120,
        timebase: "monotonic",
        events: vec![
            RecordingEvent::Marker {
                frame: 1361,
                name: "sync".to_string(),
            },
            RecordingEvent::Marker {
                frame: 1361,
                name: "sync".to_string(),
            },
        ],
    };

    let decoded = decode_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

    assert_eq!(decoded.markers.len(), 1);
    assert_eq!(decoded.markers[0].frame, 1361);
}

#[test]
fn binary_recording_excludes_virtual_void_keys() {
    let snapshot = RecordingSnapshot {
        version: 1,
        fps: 60,
        timebase: "monotonic",
        events: vec![RecordingEvent::KeyDown {
            frame: 0,
            key_id: "void".to_string(),
        }],
    };

    let decoded = decode_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

    assert!(decoded.key_ids.is_empty());
    assert!(decoded.frames.is_empty());
}

#[test]
fn inspects_binary_kbdrec_as_human_readable_events_and_frames() {
    let snapshot = {
        let mut session = RecordingSession::new(10, 1000);
        session.record_input(1100, "w", true);
        session.record_input(1300, "w", false);
        session.add_marker(1200, "sync");
        session.snapshot()
    };

    let inspection = inspect_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

    assert_eq!(inspection.version, 1);
    assert_eq!(inspection.fps, 10);
    assert_eq!(inspection.key_ids, vec!["w"]);
    let serialized = serde_json::to_value(&inspection).unwrap();
    assert_eq!(serialized["keyIds"], serde_json::json!(["w"]));
    assert!(serialized.get("key_ids").is_none());
    assert_eq!(
        inspection.events,
        vec![RecordingEvent::Marker {
            frame: 2,
            name: "sync".to_string(),
        },],
    );
    assert!(inspection.frames[0].keys.is_empty());
    assert_eq!(inspection.frames[1].keys, vec!["w".to_string()]);
    assert_eq!(inspection.frames[2].keys, vec!["w".to_string()]);
    assert!(inspection.frames[3].keys.is_empty());
}

#[test]
fn parses_recording_file_times_from_fixed_name() {
    assert_eq!(
        parse_recording_file_times("1784311866993-1784311907364.kbdrec"),
        (Some(1784311866993), Some(1784311907364)),
    );
    assert_eq!(parse_recording_file_times("notes.kbdrec"), (None, None));
}

#[test]
fn lists_recording_files_recursively() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-list-test-{}",
        std::process::id()
    ));
    let nested = root.join("nested");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&nested).unwrap();

    let snapshot = {
        let mut session = RecordingSession::new(120, 1000);
        session.record_input(1008, "w", true);
        session.add_marker(1016, "sync");
        session.snapshot()
    };
    std::fs::write(
        nested.join("1000-2000.kbdrec"),
        encode_kbdrec(&snapshot).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("ignore.txt"), "not a recording").unwrap();

    let tree = list_recording_files(root.clone()).unwrap();
    let nested_node = tree
        .children
        .iter()
        .find(|node| node.name == "nested")
        .unwrap();
    let file_node = nested_node.children.first().unwrap();
    let summary = file_node.summary.as_ref().unwrap();

    assert_eq!(file_node.node_type, "file");
    assert_eq!(summary.file_name, "1000-2000.kbdrec");
    assert_eq!(summary.start_unix_ms, Some(1000));
    assert_eq!(summary.end_unix_ms, Some(2000));
    assert_eq!(summary.fps, 120);
    assert_eq!(summary.marker_count, 1);
    assert_eq!(summary.markers[0].frame, 2);
    assert_eq!(summary.markers[0].name, "sync");

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn marks_missing_recording_root_as_not_existing() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-missing-root-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);

    let tree = list_recording_files(root).unwrap();

    assert_eq!(tree.node_type, "directory");
    assert!(!tree.exists);
    assert!(tree.children.is_empty());
}

#[test]
fn lists_recording_sidecar_metadata_with_summary() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-metadata-list-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let snapshot = {
        let mut session = RecordingSession::new(120, 1000);
        session.record_input(1008, "w", true);
        session.snapshot()
    };
    let recording_path = root.join("1000-2000.kbdrec");
    std::fs::write(&recording_path, encode_kbdrec(&snapshot).unwrap()).unwrap();
    save_recording_metadata(
        recording_path.clone(),
        RecordingMetadata {
            display_name: "Aim warmup".to_string(),
            description: "first run".to_string(),
            tags: vec!["fps".to_string(), " aim ".to_string()],
            marker_notes: vec![RecordingMarkerNote {
                frame: 12,
                name: "sync".to_string(),
                note: "first visible shot".to_string(),
            }],
        },
    )
    .unwrap();

    let tree = list_recording_files(root.clone()).unwrap();
    let summary = tree.children.first().unwrap().summary.as_ref().unwrap();

    assert_eq!(summary.metadata.display_name, "Aim warmup");
    assert_eq!(summary.metadata.description, "first run");
    assert_eq!(summary.metadata.tags, vec!["fps", "aim"]);
    assert_eq!(summary.metadata.marker_notes[0].frame, 12);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn creates_recording_folder_and_returns_updated_tree() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-folder-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);

    let tree = create_recording_folder(root.clone(), "Match 01".to_string()).unwrap();

    assert!(root.join("Match 01").is_dir());
    assert!(tree
        .children
        .iter()
        .any(|node| { node.name == "Match 01" && node.node_type == "directory" }));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn rejects_recording_folder_names_with_path_separators() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-folder-invalid-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);

    let error = create_recording_folder(root.clone(), "../escape".to_string()).unwrap_err();

    assert!(error.contains("folder name must not contain path separators"));
    assert!(!root.join("escape").exists());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn saves_and_reads_recording_sidecar_metadata() {
    let root = std::env::temp_dir().join(format!(
        "keyboard-display-recording-metadata-save-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let recording_path = root.join("1000-2000.kbdrec");

    save_recording_metadata(
        recording_path.clone(),
        RecordingMetadata {
            display_name: "  Session 1  ".to_string(),
            description: "  marker check  ".to_string(),
            tags: vec![" sync ".to_string(), "".to_string(), "ranked".to_string()],
            marker_notes: vec![RecordingMarkerNote {
                frame: 24,
                name: " sync ".to_string(),
                note: " line up with reload ".to_string(),
            }],
        },
    )
    .unwrap();

    let metadata = read_recording_metadata(recording_path).unwrap();

    assert_eq!(metadata.display_name, "Session 1");
    assert_eq!(metadata.description, "marker check");
    assert_eq!(metadata.tags, vec!["sync", "ranked"]);
    assert_eq!(metadata.marker_notes[0].name, "sync");
    assert_eq!(metadata.marker_notes[0].note, "line up with reload");

    let _ = std::fs::remove_dir_all(root);
}
