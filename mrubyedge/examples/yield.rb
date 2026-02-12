def got_block
  yield 1
  #b.call
  yield 2
  #b.call 2
end

got_block

# got_block do |x|
#   puts "Got block with #{x}"
# end