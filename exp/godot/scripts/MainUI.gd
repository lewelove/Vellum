extends Control

@onready var scroll = $ScrollContainer
@onready var grid = $ScrollContainer/GridContainer

var detail_overlay: PanelContainer

func _ready():
	anchor_right = 1.0
	anchor_bottom = 1.0
	offset_right = 0
	offset_bottom = 0
	
	if scroll:
		scroll.anchor_right = 1.0
		scroll.anchor_bottom = 1.0
		scroll.offset_right = 0
		scroll.offset_bottom = 0
	
	_create_detail_overlay()
	VellumClient.library_received.connect(_on_library_received)
	
	if VellumClient.albums_cache.size() > 0:
		_on_library_received(VellumClient.albums_cache)

func _on_library_received(albums: Array):
	for child in grid.get_children():
		child.queue_free()
		
	for album in albums:
		var card := _create_album_card()
		grid.add_child(card)
		card.setup(album)
		card.clicked.connect(_on_album_selected)

func _on_album_selected(album: Dictionary):
	detail_overlay.show()
	var label: Label = detail_overlay.get_node("VBox/Title")
	label.text = album.get("ALBUM", "")

func _create_detail_overlay():
	detail_overlay = PanelContainer.new()
	detail_overlay.visible = false
	detail_overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	
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
		200,
		260
	)
	
	var style := StyleBoxFlat.new()
	style.bg_color = Color(
		0.15,
		0.15,
		0.15
	)
	style.set_corner_radius_all(4)
	panel.add_theme_stylebox_override("panel", style)
	
	var vbox := VBoxContainer.new()
	vbox.name = "VBox"
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	vbox.add_theme_constant_override("separation", 8)
	
	var cover_container := AspectRatioContainer.new()
	cover_container.name = "CoverContainer"
	cover_container.ratio = 1.0
	cover_container.stretch_mode = AspectRatioContainer.STRETCH_FIT
	cover_container.custom_minimum_size = Vector2(
		190,
		190
	)
	
	var cover_rect := TextureRect.new()
	cover_rect.name = "CoverRect"
	cover_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	cover_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
	cover_rect.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cover_rect.size_flags_vertical = Control.SIZE_EXPAND_FILL
	
	var text_container := VBoxContainer.new()
	text_container.name = "TextContainer"
	text_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	text_container.add_theme_constant_override("separation", 2)
	text_container.offset_left = 8
	text_container.offset_right = -8
	
	var title := Label.new()
	title.name = "TitleLabel"
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	title.max_lines_visible = 2
	title.add_theme_font_size_override("font_size", 14)
	
	var artist := Label.new()
	artist.name = "ArtistLabel"
	artist.modulate = Color(
		0.7,
		0.7,
		0.7
	)
	artist.add_theme_font_size_override("font_size", 12)
	artist.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	
	cover_container.add_child(cover_rect)
	text_container.add_child(title)
	text_container.add_child(artist)
	vbox.add_child(cover_container)
	vbox.add_child(text_container)
	panel.add_child(vbox)
	
	panel.set_script(load("res://scripts/AlbumCard.gd"))
	
	return panel
