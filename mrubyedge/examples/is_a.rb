class X 
end

module Z
end

module W
  include Z
end

class Y < X
  include W
end

class V
end

p W.ancestors
p Y.ancestors

o = Y.new
p o.is_a?(X)   # => true
p o.is_a?(Y)   # => true
p o.is_a?(Z)   # => true
p o.is_a?(W)   # => true
p o.is_a?(V)   # => false