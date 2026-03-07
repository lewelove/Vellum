extends Node

signal connection_established
signal library_received(albums_data: Array)
signal album_updated(album_data: Dictionary)
signal player_status_received(status: Dictionary)

var socket := WebSocketPeer.new()
var url := "ws://127.0.0.1:8000/ws"
var albums_cache: Array = []
var last_state: int = -1

func _ready():
	socket.set_inbound_buffer_size(1024 * 1024 * 15)
	socket.set_outbound_buffer_size(1024 * 1024 * 15)
	socket.connect_to_url(url)

func _process(_delta):
	socket.poll()
	var state = socket.get_ready_state()
	
	if state != last_state:
		var state_names := [
			"CONNECTING",
			"OPEN",
			"CLOSING",
			"CLOSED"
		]
		print("VellumClient: State is now ", state_names[state])
		last_state = state
		if state == WebSocketPeer.STATE_OPEN:
			connection_established.emit()

	if state == WebSocketPeer.STATE_OPEN:
		while socket.get_available_packet_count() > 0:
			var packet = socket.get_packet()
			_handle_message(packet.get_string_from_utf8())
	
	elif state == WebSocketPeer.STATE_CLOSED:
		set_process(false)
		await get_tree().create_timer(2.0).timeout
		set_process(true)
		socket.connect_to_url(url)

func _handle_message(json_string: String):
	var json = JSON.parse_string(json_string)
	if not json or not json is Dictionary:
		return

	var msg_type = json.get("type", "")
	
	match msg_type:
		"INIT":
			albums_cache = json.get("data", [])
			library_received.emit(albums_cache)
		"UPDATE":
			album_updated.emit(json.get("payload", {}))
		"MPD_STATUS":
			player_status_received.emit(json)

func send_command(type: String, payload: Dictionary = {}):
	if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		var msg := {
			"type": type,
			"payload": payload
		}
		socket.send_text(JSON.stringify(msg))
