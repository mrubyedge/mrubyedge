module TestModule
  def module_method
    42
  end
end

class MyClass
  include TestModule
end

obj = MyClass.new
puts obj.module_method # Output: 42