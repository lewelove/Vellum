extends Control

var album_grid: Control
var detail_overlay: PanelContainer

func _ready():
	anchor_right = 1.0
	anchor_bottom = 1.0
	offset_right = 0
	offset_bottom = 0
	
	var bg = ColorRect.new()
	bg.color = Color("#323232")
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)
	
	album_grid = load("res://scripts/AlbumGrid.gd").new()
	album_grid.name = "AlbumGrid"
	album_grid.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(album_grid)
	
	_create_detail_overlay()
	VellumClient.library_received.connect(_on_library_received)
	
	if VellumClient.albums_cache.size() > 0:
		_on_library_received(VellumClient.albums_cache)

func _on_library_received(albums: Array):
	album_grid.setup(albums)

func _on_album_selected(album: Dictionary):
	detail_overlay.show()
	var label: Label = detail_overlay.get_node("VBox/Title")
	label.text = album.get("ALBUM", "")

func _create_detail_overlay():
	detail_overlay = PanelContainer.new()
	detail_overlay.visible = false
	detail_overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	detail_overlay.z_index = 10
	
	var style := StyleBoxFlat.new()
	style.bg_color = Color(
		0.1,
		0.1,
		0.1,
		0.9
	)
	detail_overlay.add_theme_stylebox_override("panel", style)
	
	var vbox := VBoxContainer.new()
	vbox.name = "VBox"
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	
	var title := Label.new()
	title.name = "Title"
	title.add_theme_font_size_override("font_size", 32)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	
	var close_btn := Button.new()
	close_btn.text = "CLOSE"
	close_btn.pressed.connect(func(): detail_overlay.hide())
	
	vbox.add_child(title)
	vbox.add_child(close_btn)
	detail_overlay.add_child(vbox)
	add_child(detail_overlay)

func _create_album_card() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(
		190,
		250
	)
	
	var panel_style := StyleBoxEmpty.new()
	panel.add_theme_stylebox_override("panel", panel_style)
	
	var vbox := VBoxContainer.new()
	vbox.name = "VBox"
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	vbox.add_theme_constant_override("separation", 0)
	
	var cover_outer := PanelContainer.new()
	cover_outer.name = "CoverContainer"
	cover_outer.clip_contents = false
	cover_outer.custom_minimum_size = Vector2(
		190,
		190
	)
	
	var cover_style := StyleBoxEmpty.new()
	cover_outer.add_theme_stylebox_override("panel", cover_style)
	
	var cover_rect := TextureRect.new()
	cover_rect.name = "CoverRect"
	cover_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	cover_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
	cover_rect.texture_filter = TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
	cover_rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	
	var text_container := VBoxContainer.new()
	text_container.name = "TextContainer"
	text_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	text_container.add_theme_constant_override("separation", 2)
	
	var spacer = Control.new()
	spacer.custom_minimum_size.y = 11
	
	var title := Label.new()
	title.name = "TitleLabel"
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	title.max_lines_visible = 1
	title.add_theme_font_size_override("font_size", 14)
	title.add_theme_color_override("font_color", Color("#FFFFFF"))
	
	var artist := Label.new()
	artist.name = "ArtistLabel"
	artist.add_theme_font_size_override("font_size", 12)
	artist.add_theme_color_override("font_color", Color("#CCCCCC"))
	artist.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	
	cover_outer.add_child(cover_rect)
	text_container.add_child(spacer)
	text_container.add_child(title)
	text_container.add_child(artist)
	vbox.add_child(cover_outer)
	vbox.add_child(text_container)
	panel.add_child(vbox)
	
	panel.set_script(load("res://scripts/AlbumCard.gd"))
	
	return panel
