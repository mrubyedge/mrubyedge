# mruby-math

A Rust implementation of the Math module for mruby/edge.

## Features

This crate provides mathematical functions and constants for mruby/edge:

### Constants

- `Math::PI` - π (pi)
- `Math::E` - e (Euler's number)

### Trigonometric Functions

- `Math.sin(x)` - Sine
- `Math.cos(x)` - Cosine
- `Math.tan(x)` - Tangent
- `Math.asin(x)` - Arcsine
- `Math.acos(x)` - Arccosine
- `Math.atan(x)` - Arctangent
- `Math.atan2(y, x)` - Arctangent of y/x

### Hyperbolic Functions

- `Math.sinh(x)` - Hyperbolic sine
- `Math.cosh(x)` - Hyperbolic cosine
- `Math.tanh(x)` - Hyperbolic tangent
- `Math.asinh(x)` - Inverse hyperbolic sine
- `Math.acosh(x)` - Inverse hyperbolic cosine
- `Math.atanh(x)` - Inverse hyperbolic tangent

### Exponential and Logarithmic Functions

- `Math.exp(x)` - e^x
- `Math.log(x)` - Natural logarithm (ln)
- `Math.log(x, base)` - Logarithm with custom base
- `Math.log10(x)` - Base-10 logarithm
- `Math.log2(x)` - Base-2 logarithm

### Root Functions

- `Math.sqrt(x)` - Square root
- `Math.cbrt(x)` - Cube root

### Other Functions

- `Math.hypot(x, y)` - Hypotenuse (√(x² + y²))
- `Math.ldexp(fraction, exponent)` - fraction × 2^exponent
- `Math.erf(x)` - Error function
- `Math.erfc(x)` - Complementary error function

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mruby-math = "0.1.0"
mrubyedge = "1.1.1"
```

Initialize the Math module in your VM:

```rust
use mruby_math::init_math;
use mrubyedge::yamrb::vm::VM;

let mut vm = VM::open(&mut rite);
init_math(&mut vm);
```

## Example

```ruby
# Calculate sine wave
x = Math::PI / 4
y = Math.sin(x)

# Calculate distance
distance = Math.hypot(3, 4)  # => 5.0

# Exponential growth
result = Math.exp(2)  # => e^2
```

## License

BSD-3-Clause
