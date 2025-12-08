class Test1
  def hello
    123
  end
end

class Test2
  def hello
    456
  end
end

# class Test3 < Test1
#   def hello
#     super + 1
#   end
# end

def hello
  789
end

puts Test1.new.hello
puts Test2.new.hello
puts hello
# puts Test3.new.hello