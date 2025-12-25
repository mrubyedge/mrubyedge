blk = proc {
  return p(1234)
}

def inner(&b)
  b.call
  return :unreachable
end

def outer(&b)
  inner(&b)
  return :unreachable
end

# ???
p outer(&blk)