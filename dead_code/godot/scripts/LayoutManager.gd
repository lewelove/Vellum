extends RefCounted

var container_width: float = 0.0
var gap_x: float = 30.0
var gap_y: float = 16.0
var card_size: float = 190.0
var text_gap: float = 11.0
var line_height_title: float = 16.0
var line_height_artist: float = 14.0
var text_gap_lesser: float = 2.0

var row_height: float:
	get:
		return gap_y + card_size + text_gap + line_height_title + text_gap_lesser + line_height_artist

var cols: int:
	get:
		return int(max(1.0, floor((container_width - 40.0 + gap_x) / (card_size + gap_x))))

var grid_width: float:
	get:
		return (float(cols) * card_size) + (float(cols - 1) * gap_x)

func get_row_y(index: int) -> float:
	return float(index) * row_height

func get_visible_indices(scroll_y: float, viewport_height: float, row_count: int) -> Dictionary:
	var buffer = 2
	var start = int(floor(scroll_y / row_height)) - buffer
	var end = int(ceil((scroll_y + viewport_height) / row_height)) + buffer
	return {
		"start": int(max(0, start)),
		"end": int(min(max(0, row_count - 1), end))
	}
