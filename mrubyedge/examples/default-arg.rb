def incr(times, state=[0])
  return if times == 0
  p "state: #{state.inspect} #{state.object_id}"
  state[0] += 1
  incr(times - 1, state)
  state
end

p incr(3)