class Foo
  def method_missing(name, *args, **kwargs)
    puts "Called #{name} with #{args.inspect}, #{kwargs.inspect}"
  end
end

foo = Foo.new
foo.bar(1, 2, 3, a: 4, b: 5)

class Bar; end

bar = Bar.new
bar.baz(10, 20, x: 30)