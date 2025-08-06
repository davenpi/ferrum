"""
🦀 Ferrum Demo: Rust-Powered Python Performance
"""

import time
import ferrum


def benchmark_function(func, *args, iterations=1, name="Function"):
    """Benchmark a function and return the average time per call"""
    start = time.perf_counter()
    
    for _ in range(iterations):
        result = func(*args)
    
    end = time.perf_counter()
    avg_time = (end - start) / iterations
    
    print(f"  {name}: {avg_time:.6f}s per call ({iterations} iterations)")
    return result, avg_time


def python_sum_large_vector(numbers):
    """Pure Python version for comparison"""
    return sum(numbers)


def python_fibonacci(n):
    """Pure Python fibonacci for comparison"""
    if n <= 1:
        return n
    return python_fibonacci(n - 1) + python_fibonacci(n - 2)


def python_find_primes(limit):
    """Pure Python sieve of eratosthenes"""
    if limit < 2:
        return []
    
    sieve = [True] * (limit + 1)
    sieve[0] = sieve[1] = False
    
    for i in range(2, int(limit**0.5) + 1):
        if sieve[i]:
            for j in range(i*i, limit + 1, i):
                sieve[j] = False
    
    return [i for i, is_prime in enumerate(sieve) if is_prime]


def python_analyze_text(text):
    """Pure Python text analysis"""
    lines = len(text.splitlines())
    words = len(text.split())
    chars = len(text)
    return (lines, words, chars)


def main():
    print("🦀 Ferrum Demo: Rust-Powered Python Performance")
    print("=" * 60)
    
    # Basic functionality test
    print("\n1. Basic Functions:")
    print(f"   Greeting: {ferrum.hello_world()}")
    print(f"   Addition: 42 + 58 = {ferrum.add(42, 58)}")
    
    # Performance comparison: Large vector sum
    print("\n2. Large Vector Sum (1 million integers):")
    large_numbers = list(range(1_000_000))
    
    rust_result, rust_time = benchmark_function(
        ferrum.sum_large_vector, large_numbers, 
        iterations=10, name="Rust"
    )
    
    python_result, python_time = benchmark_function(
        python_sum_large_vector, large_numbers, 
        iterations=10, name="Python"
    )
    
    print(f"   Results match: {rust_result == python_result}")
    print(f"   Speedup: {python_time / rust_time:.1f}x faster with Rust")
    
    # Performance comparison: Fibonacci
    print("\n3. Fibonacci Calculation (n=35):")
    fib_n = 35
    
    rust_result, rust_time = benchmark_function(
        ferrum.fibonacci, fib_n, 
        name="Rust"
    )
    
    python_result, python_time = benchmark_function(
        python_fibonacci, fib_n, 
        name="Python"
    )
    
    print(f"   Results match: {rust_result == python_result}")
    print(f"   Speedup: {python_time / rust_time:.1f}x faster with Rust")
    
    # Performance comparison: Prime finding
    print("\n4. Prime Number Generation (up to 100,000):")
    limit = 100_000
    
    rust_result, rust_time = benchmark_function(
        ferrum.find_primes, limit, 
        name="Rust"
    )
    
    python_result, python_time = benchmark_function(
        python_find_primes, limit, 
        name="Python"
    )
    
    print(f"   Found {len(rust_result)} primes")
    print(f"   Results match: {rust_result == python_result}")
    print(f"   Speedup: {python_time / rust_time:.1f}x faster with Rust")
    
    # Text analysis demo
    print("\n5. Text Analysis:")
    sample_text = """
    The quick brown fox jumps over the lazy dog.
    This is a sample text for analysis.
    It contains multiple lines and words.
    Perfect for testing our Rust-powered text analyzer!
    """ * 1000  # Make it bigger for timing
    
    rust_result, rust_time = benchmark_function(
        ferrum.analyze_text, sample_text, 
        iterations=100, name="Rust"
    )
    
    python_result, python_time = benchmark_function(
        python_analyze_text, sample_text, 
        iterations=100, name="Python"
    )
    
    lines, words, chars = rust_result
    print(f"   Text stats: {lines} lines, {words} words, {chars} characters")
    print(f"   Results match: {rust_result == python_result}")
    print(f"   Speedup: {python_time / rust_time:.1f}x faster with Rust")
    
    print("\n" + "=" * 60)
    print("🚀 Demo complete! Rust + Python = ❤️")


if __name__ == "__main__":
    main() 