# Components

前端组件按“业务面板组合基础组件”的方式组织，避免页面里重复堆叠长串样式类。

- `BasePanel` / `BasePanelHeader`：统一玻璃卡片和标题区。
- `BaseFormRow` / `BaseRange` / `BaseSelect` / `BaseToggleRow`：统一表单控件布局。
- `BaseControlGrid`：统一颜色、按钮组等控件网格。
- `ColorPicker`：颜色交互组件。弹层通过 `Teleport` 挂到 `body`，定位由 `domain/colorPopover.ts` 计算，避免被页面滚动容器或深层嵌套裁剪。

业务组件应优先组合这些基础组件，再承载自己的配置读写逻辑。
