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

p W.ancestors
p Y.ancestors