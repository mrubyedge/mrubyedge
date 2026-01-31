def splat_it(x, y, *args)
  args.each do |arg|
    p arg
  end
end

splat_it(10, 20, 30)
splat_it(10, 20, 30, 40, 50)