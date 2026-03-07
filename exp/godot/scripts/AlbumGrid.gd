extends Control

var layout = load("res://scripts/LayoutManager.gd").new()
var scroll = load("res://scripts/ScrollEngine.gd").new()

var albums: Array = []
var rows_pool: Dictionary = {}
var active_rows: Dictionary = {}

@onready var viewport_container = SubViewportContainer.new()
@onready var viewport = SubViewport.new()
@onready var grid_root = Control.new()

func _ready():
	clip_contents = true
	
	viewport_container.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	viewport_container.stretch = false
	viewport_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	viewport_container.texture_filter = TEXTURE_FILTER_LINEAR
	add_child(viewport_container)
	
	viewport.transparent_bg = true
	viewport.gui_snap_controls_to_pixels = false
	viewport.msaa_2d = Viewport.MSAA_4X
	viewport.canvas_item_default_texture_filter = Viewport.DEFAULT_CANVAS_ITEM_TEXTURE_FILTER_LINEAR
	viewport_container.add_child(viewport)
	
	grid_root.name = "GridRoot"
	grid_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	viewport.add_child(grid_root)
	
	set_process_unhandled_input(true)
	scroll.dpr = DisplayServer.screen_get_max_scale()

func setup(data: Array):
	albums = data
	_refresh_grid()

func _process(delta):
	if albums.is_empty():
		return
		
	var dpr = scroll.dpr
	var target_viewport_size = Vector2i(size * dpr)
	
	if viewport.size != target_viewport_size:
		viewport.size = target_viewport_size
		viewport_container.scale = Vector2(1.0 / dpr, 1.0 / dpr)
		
	layout.container_width = size.x
	var row_count = int(ceil(float(albums.size()) / layout.cols))
	var max_slots = max(0.0, float(row_count) - (size.y / layout.row_height))
	
	scroll.update(delta, layout.row_height)
	
	var x_offset = (size.x - layout.grid_width) / 2.0
	var offset = Vector2(x_offset, -scroll.current_y) * dpr
	viewport.canvas_transform = Transform2D(0.0, Vector2(dpr, dpr), 0.0, offset)
	
	_update_virtual_rows(row_count)

func _update_virtual_rows(row_count: int):
	var range = layout.get_visible_indices(scroll.current_y, size.y, row_count)
	
	var needed_indices = []
	for i in range(range.start, range.end + 1):
		needed_indices.append(i)
		
	for idx in active_rows.keys():
		if not idx in needed_indices:
			var row = active_rows[idx]
			row.hide()
			rows_pool[row] = true
			active_rows.erase(idx)
			
	for idx in needed_indices:
		if not idx in active_rows:
			var row = _get_row_from_pool()
			_bind_row_data(row, idx)
			row.position.y = layout.get_row_y(idx)
			row.show()
			active_rows[idx] = row

func _get_row_from_pool() -> HBoxContainer:
	if rows_pool.is_empty():
		var row = HBoxContainer.new()
		row.add_theme_constant_override("separation", int(layout.gap_x))
		row.mouse_filter = Control.MOUSE_FILTER_IGNORE
		grid_root.add_child(row)
		return row
	var row = rows_pool.keys()[0]
	rows_pool.erase(row)
	return row

func _bind_row_data(row_node: HBoxContainer, row_index: int):
	var start_idx = row_index * layout.cols
	for i in range(layout.cols):
		var album_idx = start_idx + i
		var card: PanelContainer
		
		if i < row_node.get_child_count():
			card = row_node.get_child(i)
		else:
			card = _create_card_instance()
			row_node.add_child(card)
			
		if album_idx < albums.size():
			card.show()
			card.setup(albums[album_idx])
		else:
			card.hide()

func _create_card_instance() -> PanelContainer:
	var main_ui = get_tree().current_scene
	return main_ui._create_album_card()

func _gui_input(event):
	if event is InputEventMouseButton:
		var row_count = int(ceil(float(albums.size()) / layout.cols))
		var max_slots = max(0.0, float(row_count) - (size.y / layout.row_height))
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			scroll.handle_wheel(-40.0, max_slots)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			scroll.handle_wheel(40.0, max_slots)

func _unhandled_input(event):
	if event is InputEventKey and event.pressed:
		var row_count = int(ceil(float(albums.size()) / layout.cols))
		var max_slots = max(0.0, float(row_count) - (size.y / layout.row_height))
		match event.keycode:
			KEY_J, KEY_DOWN:
				scroll.target_slot = clamp(scroll.target_slot + 1.0, 0, max_slots)
			KEY_K, KEY_UP:
				scroll.target_slot = clamp(scroll.target_slot - 1.0, 0, max_slots)

func _refresh_grid():
	for idx in active_rows.keys():
		var row = active_rows[idx]
		row.hide()
		rows_pool[row] = true
	active_rows.clear()
	scroll.sync_to_slot(0)
