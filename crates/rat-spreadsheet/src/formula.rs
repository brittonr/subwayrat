//! Formula engine: parsing, evaluation, and dependency tracking.
//!
//! Formulas start with `=` and support arithmetic operators, cell references,
//! range references, and function calls. The engine includes:
//! - Tokenizer and recursive descent parser producing an AST
//! - Tree-walk evaluator with cell reference resolution
//! - Built-in functions: SUM, AVG, MIN, MAX, COUNT, IF
//! - Custom function registry for user-defined functions
//! - Dependency graph with topological sort for recalculation
//! - Circular reference detection

use crate::cell::{CellAddr, CellError, CellValue};
use std::collections::{HashMap, HashSet};

// AST types
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    CellRef(CellAddr),
    Range(CellAddr, CellAddr),
    BinaryOp {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

// Tokenizer types
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    CellRef(CellAddr),
    Operator(Op),
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Function(String),
    Eof,
}

struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let number_str: String = self.input[start..self.pos].iter().collect();
        number_str.parse().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> String {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        self.input[start..self.pos].iter().collect()
    }

    fn is_cell_reference(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        // Must start with letters (column)
        while i < chars.len() && chars[i].is_ascii_alphabetic() {
            i += 1;
        }

        if i == 0 {
            return false;
        }

        // Must have digits (row)
        let digit_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }

        i == chars.len() && i > digit_start
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.peek() {
            None => Token::Eof,
            Some(ch) => match ch {
                '(' => {
                    self.advance();
                    Token::LeftParen
                }
                ')' => {
                    self.advance();
                    Token::RightParen
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                ':' => {
                    self.advance();
                    Token::Colon
                }
                '+' => {
                    self.advance();
                    Token::Operator(Op::Add)
                }
                '-' => {
                    self.advance();
                    Token::Operator(Op::Sub)
                }
                '*' => {
                    self.advance();
                    Token::Operator(Op::Mul)
                }
                '/' => {
                    self.advance();
                    Token::Operator(Op::Div)
                }
                '%' => {
                    self.advance();
                    Token::Operator(Op::Mod)
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::Operator(Op::Gte)
                    } else {
                        Token::Operator(Op::Gt)
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::Operator(Op::Lte)
                    } else if self.peek() == Some('>') {
                        self.advance();
                        Token::Operator(Op::Neq)
                    } else {
                        Token::Operator(Op::Lt)
                    }
                }
                '=' => {
                    self.advance();
                    Token::Operator(Op::Eq)
                }
                _ if ch.is_ascii_digit() => Token::Number(self.read_number()),
                _ if ch.is_ascii_alphabetic() => {
                    let identifier = self.read_identifier();

                    // Check if this is a function (followed by '(')
                    self.skip_whitespace();
                    if self.peek() == Some('(') {
                        Token::Function(identifier.to_uppercase())
                    } else if Self::is_cell_reference(&identifier) {
                        if let Ok(addr) = identifier.parse() {
                            Token::CellRef(addr)
                        } else {
                            // Invalid cell reference, treat as Eof for now
                            Token::Eof
                        }
                    } else {
                        // Unknown identifier, treat as Eof for now
                        Token::Eof
                    }
                }
                _ => {
                    // Unknown character, skip it
                    self.advance();
                    self.next_token()
                }
            },
        }
    }
}

// Parser
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = Vec::new();

        loop {
            let token = tokenizer.next_token();
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        self.peek()
    }

    fn expect(&mut self, expected: Token) -> Result<(), CellError> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(CellError::ParseError)
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, CellError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, CellError> {
        let mut left = self.parse_additive()?;

        while let Token::Operator(op) = self.peek() {
            match op {
                Op::Gt | Op::Lt | Op::Gte | Op::Lte | Op::Eq | Op::Neq => {
                    let op = op.clone();
                    self.advance();
                    let right = self.parse_additive()?;
                    left = Expr::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, CellError> {
        let mut left = self.parse_multiplicative()?;

        while let Token::Operator(op) = self.peek() {
            match op {
                Op::Add | Op::Sub => {
                    let op = op.clone();
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, CellError> {
        let mut left = self.parse_atom()?;

        while let Token::Operator(op) = self.peek() {
            match op {
                Op::Mul | Op::Div | Op::Mod => {
                    let op = op.clone();
                    self.advance();
                    let right = self.parse_atom()?;
                    left = Expr::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<Expr, CellError> {
        match self.peek() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::CellRef(addr) => {
                let addr = *addr;
                self.advance();

                // Check if this is a range (A1:B2)
                if matches!(self.peek(), Token::Colon) {
                    self.advance(); // consume ':'
                    if let Token::CellRef(end_addr) = self.peek() {
                        let end_addr = *end_addr;
                        self.advance();
                        Ok(Expr::Range(addr, end_addr))
                    } else {
                        Err(CellError::ParseError)
                    }
                } else {
                    Ok(Expr::CellRef(addr))
                }
            }
            Token::Function(name) => {
                let name = name.clone();
                self.advance();
                self.expect(Token::LeftParen)?;

                let mut args = Vec::new();
                if !matches!(self.peek(), Token::RightParen) {
                    args.push(self.parse_expression()?);

                    while matches!(self.peek(), Token::Comma) {
                        self.advance(); // consume ','
                        args.push(self.parse_expression()?);
                    }
                }

                self.expect(Token::RightParen)?;
                Ok(Expr::FunctionCall { name, args })
            }
            Token::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(CellError::ParseError),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, CellError> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression()?;

    // Ensure we consumed all tokens
    if !matches!(parser.peek(), Token::Eof) {
        return Err(CellError::ParseError);
    }

    Ok(expr)
}

// Mock Grid trait for dependency on cell module
pub trait Grid {
    fn get(&self, addr: CellAddr) -> &CellValue;
}

// Function registry
type FunctionDef = Box<dyn Fn(&[CellValue]) -> CellValue>;

pub struct FunctionRegistry {
    functions: HashMap<String, FunctionDef>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register built-in functions
        registry.register("SUM", |args| {
            let mut sum = 0.0;
            for arg in args {
                match arg {
                    CellValue::Number(n) => sum += n,
                    CellValue::Empty => sum += 0.0,
                    _ => {}
                }
            }
            CellValue::Number(sum)
        });

        registry.register("AVG", |args| {
            let mut sum = 0.0;
            let mut count = 0;
            for arg in args {
                match arg {
                    CellValue::Number(n) => {
                        sum += n;
                        count += 1;
                    }
                    CellValue::Empty => {
                        sum += 0.0;
                        count += 1;
                    }
                    _ => {}
                }
            }
            if count > 0 {
                CellValue::Number(sum / count as f64)
            } else {
                CellValue::Number(0.0)
            }
        });

        registry.register("MIN", |args| {
            let mut min = f64::INFINITY;
            let mut found_number = false;
            for arg in args {
                match arg {
                    CellValue::Number(n) => {
                        min = min.min(*n);
                        found_number = true;
                    }
                    CellValue::Empty => {
                        min = min.min(0.0);
                        found_number = true;
                    }
                    _ => {}
                }
            }
            if found_number {
                CellValue::Number(min)
            } else {
                CellValue::Number(0.0)
            }
        });

        registry.register("MAX", |args| {
            let mut max = f64::NEG_INFINITY;
            let mut found_number = false;
            for arg in args {
                match arg {
                    CellValue::Number(n) => {
                        max = max.max(*n);
                        found_number = true;
                    }
                    CellValue::Empty => {
                        max = max.max(0.0);
                        found_number = true;
                    }
                    _ => {}
                }
            }
            if found_number {
                CellValue::Number(max)
            } else {
                CellValue::Number(0.0)
            }
        });

        registry.register("COUNT", |args| {
            let mut count = 0;
            for arg in args {
                match arg {
                    CellValue::Empty => {}
                    _ => count += 1,
                }
            }
            CellValue::Number(count as f64)
        });

        registry.register("IF", |args| {
            if args.len() != 3 {
                return CellValue::Error(CellError::ValueError);
            }

            let condition = match &args[0] {
                CellValue::Number(n) => *n != 0.0,
                CellValue::Boolean(b) => *b,
                CellValue::Empty => false,
                _ => return CellValue::Error(CellError::ValueError),
            };

            if condition {
                args[1].clone()
            } else {
                args[2].clone()
            }
        });

        registry
    }
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[CellValue]) -> CellValue + 'static,
    {
        self.functions.insert(name.to_uppercase(), Box::new(f));
    }

    fn call(&self, name: &str, args: &[CellValue]) -> CellValue {
        if let Some(func) = self.functions.get(&name.to_uppercase()) {
            func(args)
        } else {
            CellValue::Error(CellError::ValueError)
        }
    }
}

// Evaluator
fn collect_range_values(start: CellAddr, end: CellAddr, grid: &dyn Grid) -> Vec<CellValue> {
    let mut values = Vec::new();
    let start_col = start.col.min(end.col);
    let end_col = start.col.max(end.col);
    let start_row = start.row.min(end.row);
    let end_row = start.row.max(end.row);

    for row in start_row..=end_row {
        for col in start_col..=end_col {
            let addr = CellAddr { col, row };
            let value = grid.get(addr);
            values.push(resolve_cell_value(value));
        }
    }

    values
}

fn resolve_cell_value(value: &CellValue) -> CellValue {
    match value {
        CellValue::Formula { cached, .. } => (**cached).clone(),
        CellValue::Text(s) => {
            if let Ok(n) = s.parse::<f64>() {
                CellValue::Number(n)
            } else {
                CellValue::Error(CellError::ValueError)
            }
        }
        other => other.clone(),
    }
}

pub fn evaluate(expr: &Expr, grid: &dyn Grid) -> CellValue {
    evaluate_with_registry(expr, grid, &FunctionRegistry::default())
}

pub fn evaluate_with_registry(
    expr: &Expr,
    grid: &dyn Grid,
    registry: &FunctionRegistry,
) -> CellValue {
    match expr {
        Expr::Number(n) => CellValue::Number(*n),

        Expr::CellRef(addr) => {
            let value = grid.get(*addr);
            resolve_cell_value(value)
        }

        Expr::Range(_, _) => {
            // Range by itself is invalid
            CellValue::Error(CellError::ValueError)
        }

        Expr::BinaryOp { op, left, right } => {
            let left_val = evaluate_with_registry(left, grid, registry);
            let right_val = evaluate_with_registry(right, grid, registry);

            match (&left_val, &right_val) {
                (CellValue::Number(l), CellValue::Number(r)) => match op {
                    Op::Add => CellValue::Number(l + r),
                    Op::Sub => CellValue::Number(l - r),
                    Op::Mul => CellValue::Number(l * r),
                    Op::Div => {
                        if *r == 0.0 {
                            CellValue::Error(CellError::DivByZero)
                        } else {
                            CellValue::Number(l / r)
                        }
                    }
                    Op::Mod => {
                        if *r == 0.0 {
                            CellValue::Error(CellError::DivByZero)
                        } else {
                            CellValue::Number(l % r)
                        }
                    }
                    Op::Gt => CellValue::Boolean(l > r),
                    Op::Lt => CellValue::Boolean(l < r),
                    Op::Gte => CellValue::Boolean(l >= r),
                    Op::Lte => CellValue::Boolean(l <= r),
                    Op::Eq => CellValue::Boolean(l == r),
                    Op::Neq => CellValue::Boolean(l != r),
                },
                (CellValue::Error(e), _) => CellValue::Error(e.clone()),
                (_, CellValue::Error(e)) => CellValue::Error(e.clone()),
                _ => CellValue::Error(CellError::ValueError),
            }
        }

        Expr::FunctionCall { name, args } => {
            let mut arg_values = Vec::new();

            for arg in args {
                match arg {
                    Expr::Range(start, end) => {
                        arg_values.extend(collect_range_values(*start, *end, grid));
                    }
                    _ => {
                        let val = evaluate_with_registry(arg, grid, registry);
                        arg_values.push(val);
                    }
                }
            }

            registry.call(name, &arg_values)
        }
    }
}

// Dependency tracking
pub struct DependencyGraph {
    /// Maps cell -> cells that depend on it
    dependents: HashMap<CellAddr, Vec<CellAddr>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependents: HashMap::new(),
        }
    }

    pub fn update_deps(&mut self, cell: CellAddr, expr: &Expr) {
        // Remove old dependencies
        for (_, deps) in self.dependents.iter_mut() {
            deps.retain(|&dep| dep != cell);
        }

        // Add new dependencies
        let dependencies = extract_dependencies(expr);
        for dep in dependencies {
            self.dependents.entry(dep).or_default().push(cell);
        }
    }

    pub fn get_dependents(&self, cell: CellAddr) -> Vec<CellAddr> {
        self.dependents.get(&cell).cloned().unwrap_or_default()
    }

    pub fn get_recalc_order(&self, changed: CellAddr) -> Result<Vec<CellAddr>, CellError> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut result = Vec::new();

        self.dfs(changed, &mut visited, &mut temp_visited, &mut result)?;

        Ok(result)
    }

    fn dfs(
        &self,
        cell: CellAddr,
        visited: &mut HashSet<CellAddr>,
        temp_visited: &mut HashSet<CellAddr>,
        result: &mut Vec<CellAddr>,
    ) -> Result<(), CellError> {
        if temp_visited.contains(&cell) {
            return Err(CellError::CycleError);
        }

        if visited.contains(&cell) {
            return Ok(());
        }

        temp_visited.insert(cell);

        for &dependent in &self.get_dependents(cell) {
            self.dfs(dependent, visited, temp_visited, result)?;
        }

        temp_visited.remove(&cell);
        visited.insert(cell);

        if cell
            != result.first().copied().unwrap_or(CellAddr {
                col: usize::MAX,
                row: usize::MAX,
            })
        {
            result.push(cell);
        }

        Ok(())
    }
}

fn extract_dependencies(expr: &Expr) -> Vec<CellAddr> {
    let mut deps = Vec::new();
    extract_dependencies_recursive(expr, &mut deps);
    deps
}

fn extract_dependencies_recursive(expr: &Expr, deps: &mut Vec<CellAddr>) {
    match expr {
        Expr::CellRef(addr) => deps.push(*addr),
        Expr::Range(start, end) => {
            let start_col = start.col.min(end.col);
            let end_col = start.col.max(end.col);
            let start_row = start.row.min(end.row);
            let end_row = start.row.max(end.row);

            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    deps.push(CellAddr { col, row });
                }
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            extract_dependencies_recursive(left, deps);
            extract_dependencies_recursive(right, deps);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                extract_dependencies_recursive(arg, deps);
            }
        }
        Expr::Number(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock grid for testing
    struct MockGrid {
        cells: HashMap<CellAddr, CellValue>,
    }

    impl MockGrid {
        fn new() -> Self {
            Self {
                cells: HashMap::new(),
            }
        }

        fn set(&mut self, addr: CellAddr, value: CellValue) {
            self.cells.insert(addr, value);
        }
    }

    impl Grid for MockGrid {
        fn get(&self, addr: CellAddr) -> &CellValue {
            self.cells.get(&addr).unwrap_or(&CellValue::Empty)
        }
    }

    #[test]
    fn test_parse_and_eval_simple() {
        let grid = MockGrid::new();

        let expr = parse("1+2").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(3.0));
    }

    #[test]
    fn test_precedence() {
        let grid = MockGrid::new();

        let expr = parse("2+3*4").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(14.0));

        let expr = parse("(2+3)*4").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(20.0));
    }

    #[test]
    fn test_division_by_zero() {
        let grid = MockGrid::new();

        let expr = parse("1/0").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Error(CellError::DivByZero));
    }

    #[test]
    fn test_cell_references() {
        let mut grid = MockGrid::new();
        grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(5.0)); // A1
        grid.set(CellAddr { col: 1, row: 0 }, CellValue::Number(3.0)); // B1

        let expr = parse("A1+B1").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(8.0));
    }

    #[test]
    fn test_sum_function() {
        let mut grid = MockGrid::new();
        grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(1.0)); // A1
        grid.set(CellAddr { col: 0, row: 1 }, CellValue::Number(2.0)); // A2
        grid.set(CellAddr { col: 0, row: 2 }, CellValue::Number(3.0)); // A3

        let expr = parse("SUM(A1:A3)").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(6.0));
    }

    #[test]
    fn test_count_function() {
        let mut grid = MockGrid::new();
        grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(1.0)); // A1
        grid.set(CellAddr { col: 0, row: 1 }, CellValue::Empty); // A2 (empty)
        grid.set(
            CellAddr { col: 0, row: 2 },
            CellValue::Text("hello".to_string()),
        ); // A3

        let expr = parse("COUNT(A1:A3)").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(2.0)); // Only A1 and A3 are non-empty
    }

    #[test]
    fn test_if_function() {
        let grid = MockGrid::new();

        let expr = parse("IF(1,5,10)").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(5.0));

        let expr = parse("IF(0,5,10)").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Number(10.0));
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        // A1 depends on B1 and C1
        let expr = parse("B1+C1").unwrap();
        graph.update_deps(CellAddr { col: 0, row: 0 }, &expr);

        // Check that B1 and C1 have A1 as dependent
        let deps_b1 = graph.get_dependents(CellAddr { col: 1, row: 0 });
        assert!(deps_b1.contains(&CellAddr { col: 0, row: 0 }));

        let deps_c1 = graph.get_dependents(CellAddr { col: 2, row: 0 });
        assert!(deps_c1.contains(&CellAddr { col: 0, row: 0 }));
    }

    #[test]
    fn test_recalc_order() {
        let mut graph = DependencyGraph::new();

        // A1 = B1 + C1
        let expr = parse("B1+C1").unwrap();
        graph.update_deps(CellAddr { col: 0, row: 0 }, &expr);

        // D1 = A1 * 2
        let expr = parse("A1*2").unwrap();
        graph.update_deps(CellAddr { col: 3, row: 0 }, &expr);

        let order = graph.get_recalc_order(CellAddr { col: 1, row: 0 }).unwrap(); // B1 changed

        // Should include A1 and D1 in dependency order
        assert!(order.contains(&CellAddr { col: 0, row: 0 })); // A1
        assert!(order.contains(&CellAddr { col: 3, row: 0 })); // D1
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // A1 = B1
        let expr = parse("B1").unwrap();
        graph.update_deps(CellAddr { col: 0, row: 0 }, &expr);

        // B1 = A1 (creates cycle)
        let expr = parse("A1").unwrap();
        graph.update_deps(CellAddr { col: 1, row: 0 }, &expr);

        let result = graph.get_recalc_order(CellAddr { col: 0, row: 0 });
        assert!(matches!(result, Err(CellError::CycleError)));
    }

    #[test]
    fn test_comparison_operators() {
        let grid = MockGrid::new();

        let expr = parse("5>3").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Boolean(true));

        let expr = parse("2<=2").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Boolean(true));

        let expr = parse("1<>1").unwrap();
        let result = evaluate(&expr, &grid);
        assert_eq!(result, CellValue::Boolean(false));
    }
}
