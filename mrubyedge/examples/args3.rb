def splat_it(x, y, *args, buz: -1, **kwargs)
  args.each do |arg|
    p arg
  end
  p buz: buz
  p kwargs
end

splat_it(10, 20, 30)
splat_it(10, 20, 30, 40, 50, foo: 60)
splat_it(10, 20, 30, 40, 50, foo: 60, buz: 70)