use crate::ast::ResolvedFileImports;

pub fn build_dependency_graph(file_imports: &[ResolvedFileImports]) -> Vec<Vec<bool>> {
    let mut matrix = vec![vec![false; file_imports.len()]; file_imports.len()];
    for (x, fx) in file_imports.iter().enumerate() {
        for (y, fy) in file_imports.iter().enumerate() {
            if x == y {
                matrix[x][y] = true;
                continue;
            }
            if fx.imports.contains(&fy.file.to_string_lossy().to_string()) {
                matrix[x][y] = true;
            }
        }
    }

    multiply_to_idempotency(&matrix)
}

//    a b c d
// a  1 0 1 0
// b  0 1 0 1
// c  1 0 1 0
// d  0 1 0 1

const MAX_ITERATIONS: usize = 500;

fn multiply_to_idempotency(matrix: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut result = matrix.to_owned();
    for _ in 0..MAX_ITERATIONS {
        let new_result = matrix_multiply(&result, matrix);
        if new_result == result {
            return result;
        }
        result = new_result;
    }
    panic!(
        "Failed to converge to idempotency in less than {} iterations",
        MAX_ITERATIONS
    );
}

fn matrix_multiply(m1: &[Vec<bool>], m2: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut result = vec![vec![false; m1.len()]; m1.len()];
    for i in 0..m1.len() {
        for j in 0..m1.len() {
            if i == j {
                result[i][j] = true;
                continue;
            }
            for k in 0..m1.len() {
                if m1[i][k] && m2[k][j] {
                    result[i][j] = true;
                    break;
                }
            }
        }
    }
    result
}
