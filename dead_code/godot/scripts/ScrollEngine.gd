extends RefCounted

var current_y: float = 0.0
var target_slot: float = 0.0
var wheel_accumulator: float = 0.0
var damping: float = 0.18
var threshold: float = 40.0
var dpr: float = 1.0

func update(row_height: float):
	var ideal_target_y = target_slot * row_height
	var snapped_target_y = round(ideal_target_y * dpr) / dpr
	
	var diff = snapped_target_y - current_y
	var velocity = diff * damping

	if abs(diff) < 0.01:
		current_y = snapped_target_y
	else:
		current_y += velocity

func handle_wheel(delta_y: float, max_slots: float):
	wheel_accumulator += delta_y
	
	if abs(wheel_accumulator) > threshold:
		var direction = 1.0 if wheel_accumulator > 0.0 else -1.0
		var base = round(target_slot)
		
		target_slot = clamp(base + direction, 0.0, max_slots)
		wheel_accumulator = 0.0

func sync_to_slot(slot: float):
	target_slot = slot
