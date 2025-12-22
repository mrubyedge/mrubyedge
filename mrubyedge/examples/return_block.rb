def outer
  inner do
    return 5471
  end
  return :unreachable
end

def inner
  yield
  return :unreachable
end

p outer