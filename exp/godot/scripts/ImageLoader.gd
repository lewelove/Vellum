extends Node

var cache: Dictionary = {}
var pending_requests: Dictionary = {}
var base_url := "http://127.0.0.1:8000/api/covers"

func load_album_cover(cover_hash: String, size: int, target_rect: TextureRect):
	if cover_hash.is_empty():
		return

	if cache.has(cover_hash):
		target_rect.texture = cache[cover_hash]
		return

	if pending_requests.has(cover_hash):
		pending_requests[cover_hash].append(target_rect)
		return

	pending_requests[cover_hash] = [target_rect]
	
	var http := HTTPRequest.new()
	add_child(http)
	http.request_completed.connect(_on_request_completed.bind(http, cover_hash))
	
	var url := "%s/%dpx/%s" % [
		base_url,
		size,
		cover_hash
	]
	
	print("ImageLoader: Requesting ", url)
	var err := http.request(url)
	if err != OK:
		print("ImageLoader: HTTP Request failed to start for ", cover_hash)
		pending_requests.erase(cover_hash)
		http.queue_free()

func _on_request_completed(_result: int, response_code: int, _headers: PackedStringArray, body: PackedByteArray, http: HTTPRequest, cover_hash: String):
	print("ImageLoader: Received response ", response_code, " for ", cover_hash)
	
	if response_code == 200:
		var image := Image.new()
		var err := image.load_png_from_buffer(body)
		
		if err == OK:
			var texture := ImageTexture.create_from_image(image)
			cache[cover_hash] = texture
			
			if pending_requests.has(cover_hash):
				for rect in pending_requests[cover_hash]:
					if is_instance_valid(rect):
						rect.texture = texture
		else:
			print("ImageLoader: Failed to parse image data for ", cover_hash, " Error: ", err)
	
	pending_requests.erase(cover_hash)
	http.queue_free()
