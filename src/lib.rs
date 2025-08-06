use pyo3::prelude::*;

/// A simple "Hello, world!" function that can be called from Python
#[pyfunction]
fn hello_world() -> PyResult<String> {
    Ok("Hello, world from Rust!".to_string())
}

/// Add two numbers together (your existing function, now Python-callable)
#[pyfunction]
fn add(left: u64, right: u64) -> PyResult<u64> {
    Ok(left + right)
}

/// Sum a large vector of numbers - this will show the performance difference!
#[pyfunction]
fn sum_large_vector(numbers: Vec<i64>) -> PyResult<i64> {
    Ok(numbers.iter().sum())
}

/// Calculate fibonacci number (recursive) - CPU intensive
#[pyfunction]
fn fibonacci(n: u32) -> PyResult<u64> {
    fn fib(n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => fib(n - 1) + fib(n - 2),
        }
    }
    Ok(fib(n))
}

/// Find prime numbers up to n using Sieve of Eratosthenes
#[pyfunction]
fn find_primes(limit: usize) -> PyResult<Vec<usize>> {
    if limit < 2 {
        return Ok(vec![]);
    }

    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;

    for i in 2..=((limit as f64).sqrt() as usize) {
        if sieve[i] {
            let mut j = i * i;
            while j <= limit {
                sieve[j] = false;
                j += i;
            }
        }
    }

    Ok(sieve
        .iter()
        .enumerate()
        .filter_map(|(i, &is_prime)| if is_prime { Some(i) } else { None })
        .collect())
}

/// Process text: count words, characters, lines
#[pyfunction]
fn analyze_text(text: String) -> PyResult<(usize, usize, usize)> {
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    let chars = text.chars().count();

    Ok((lines, words, chars))
}

/// A Python module implemented in Rust.
/// The name of this function must match the lib.name in Cargo.toml
#[pymodule]
fn ferrum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_world, m)?)?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(sum_large_vector, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_function(wrap_pyfunction!(find_primes, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_text, m)?)?;
    Ok(())
}
