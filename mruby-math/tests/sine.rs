extern crate mruby_math;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn test_sine_curve() {
    let code = r##"
# Sin curve ASCII art
# Draw sine wave from 0 to 4π

PI = Math::PI

# Settings
width = 60        # Width of the graph
height = 15       # Height of the graph (centered)
x_range = 4.to_f * PI  # 0 to 4π
steps = 80        # Number of points to plot

puts "Sin(x) curve from 0 to 4π"
puts "=" * width
puts ""

# Draw the curve
(0..steps).each do |i|
  x = (x_range * i) / steps
  y = Math.sin(x)
  
  # Map y (-1 to 1) to column position (0 to width-1)
  col = ((y + 1) * (width - 1) / 2).to_i
  
  # Print spaces then the marker
  line = " " * col + "*"
  
  # Add x-axis marker at y=0
  if y.abs < 0.1
    line = line.sub("*", "|")
  end
  
  # Show x value at specific points
  if i % 20 == 0
    x_label = (x / PI * 10).to_i / 10.0
    puts line + "  (x=#{x_label}π)"
  else
    puts line
  end
end

puts ""
puts "=" * width
puts "Legend: * = sin(x), | = x-axis (y≈0)"
    "##;
    let binary = mrbc_compile("sine_curve", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    assert!(vm.run().unwrap().is_nil());
}
