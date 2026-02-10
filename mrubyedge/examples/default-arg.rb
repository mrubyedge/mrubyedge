def incr(times, state)
  return if times == 0
  p "state: #{state.inspect}"
  state[0] += 1
  p "state: #{state.inspect}"
  incr(times - 1, state)
  state
end

p incr(1, [0])