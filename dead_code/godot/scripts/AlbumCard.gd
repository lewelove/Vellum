extends PanelContainer

signal clicked(data: Dictionary)

@onready var cover_rect : TextureRect = get_node("VBox/CoverContainer/CoverRect")
@onready var title_label : Label = get_node("VBox/TextContainer/TitleLabel")
@onready var artist_label : Label = get_node("VBox/TextContainer/ArtistLabel")

var album_data: Dictionary
var _is_ready: bool = false

func setup(data: Dictionary):
	album_data = data
	if _is_ready:
		_update_ui()

func _ready():
	_is_ready = true
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	gui_input.connect(_on_gui_input)
	# mouse_entered.connect(_on_mouse_entered)
	# mouse_exited.connect(_on_mouse_exited)
	
	if not album_data.is_empty():
		_update_ui()

func _update_ui():
	var info: Dictionary = album_data.get("info", {})
	var cover_hash: String = info.get("cover_hash", "")
	
	title_label.text = album_data.get("ALBUM", "Unknown Album")
	artist_label.text = album_data.get("ALBUMARTIST", "Unknown Artist")
	
	if not cover_hash.is_empty():
		ImageLoader.load_album_cover(cover_hash, 190, cover_rect)
	else:
		printerr("AlbumCard: No cover hash found for ", album_data.get("ALBUM"))

func _on_gui_input(event: InputEvent):
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
			clicked.emit(album_data)

# func _on_mouse_entered():
# 	var tween := create_tween()
# 	tween.tween_property(
# 		self,
# 		"modulate",
# 		Color(1.2, 1.2, 1.2),
# 		0.1
# 	)

# func _on_mouse_exited():
# 	var tween := create_tween()
# 	tween.tween_property(
# 		self,
# 		"modulate",
# 		Color(1, 1, 1),
# 		0.1
# 	)
