extends RefCounted

var current_y: float = 0.0
var target_slot: float = 0.0
var wheel_accumulator: float = 0.0
var damping: float = 0.18
var threshold: float = 40.0
var dpr: float = 1.0

func update(delta_time: float, row_height: float):
	var ideal_target_y = target_slot * row_height
	var snapped_target_y = round(ideal_target_y * dpr) / dpr
	var diff = snapped_target_y - current_y
	var velocity = diff * (1.0 - pow(1.0 - damping, delta_time * 60.0))
	
	if abs(diff) < (0.05 / dpr):
		current_y = snapped_target_y
	else:
		current_y += velocity

func handle_wheel(delta_y: float, max_slots: float):
	wheel_accumulator += delta_y
	if abs(wheel_accumulator) > threshold:
		var direction = 1.0 if wheel_accumulator > 0 else -1.0
		target_slot = clamp(round(target_slot) + direction, 0, max_slots)
		wheel_accumulator = 0.0

func sync_to_slot(slot: float):
	target_slot = slot
