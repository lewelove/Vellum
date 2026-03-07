extends Node

var cache: Dictionary = {}
var thumb_base_dir: String = ""

func _ready():
	var home = OS.get_environment("HOME")
	if home == "":
		home = OS.get_environment("USERPROFILE")
	
	thumb_base_dir = home.path_join(".vellum/thumbnails/190px")

func load_album_cover(cover_hash: String, _size: int, target_rect: TextureRect):
	if cover_hash.is_empty():
		return

	if cache.has(cover_hash):
		target_rect.texture = cache[cover_hash]
		return

	var full_path = thumb_base_dir.path_join(cover_hash + ".png")

	if not FileAccess.file_exists(full_path):
		return

	var raw_image = Image.load_from_file(full_path)
	
	if raw_image:
		var w = raw_image.get_width()
		var h = raw_image.get_height()
		
		var padded = Image.create(w + 2, h + 2, false, raw_image.get_format())
		padded.fill(Color(0, 0, 0, 0))
		padded.blit_rect(raw_image, Rect2i(0, 0, w, h), Vector2i(1, 1))
		
		padded.generate_mipmaps()
		
		var texture = ImageTexture.create_from_image(padded)
		cache[cover_hash] = texture
		target_rect.texture = texture
