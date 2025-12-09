module TestModule
  module ChildModule
    def module_method
      42
    end
  end

  class MyClass
    include ChildModule
  end
end

obj = TestModule::MyClass.new
# puts obj
puts obj.module_method # Output: 42