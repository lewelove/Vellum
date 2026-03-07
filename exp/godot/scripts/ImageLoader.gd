extends Node

var cache: Dictionary = {}
var thumb_base_dir: String = ""

func _ready():
	var home = OS.get_environment("HOME")
	if home == "":
		home = OS.get_environment("USERPROFILE")
	
	thumb_base_dir = home.path_join(".vellum/thumbnails/190px")
	print("ImageLoader: Base directory set to: ", thumb_base_dir)

func load_album_cover(cover_hash: String, _size: int, target_rect: TextureRect):
	if cover_hash.is_empty():
		return

	if cache.has(cover_hash):
		target_rect.texture = cache[cover_hash]
		return

	var full_path = thumb_base_dir.path_join(cover_hash + ".png")

	if not FileAccess.file_exists(full_path):
		printerr("ImageLoader: File not found at path: ", full_path)
		return

	var image = Image.load_from_file(full_path)
	
	if image:
		var texture = ImageTexture.create_from_image(image)
		cache[cover_hash] = texture
		target_rect.texture = texture
		print("ImageLoader: Successfully loaded: ", cover_hash)
	else:
		printerr("ImageLoader: Failed to parse image data at: ", full_path)
