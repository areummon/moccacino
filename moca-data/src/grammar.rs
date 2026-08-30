/* Context-free grammars: model, text parsing, derivations, Chomsky normal
 * form and CYK membership. Grammars are deliberately NOT state machines, so
 * they implement neither the StateMachine nor the Machine trait.
 *
 * A symbol is any string; variables are exactly the symbols that appear as
 * the left-hand side of some production, and every other body symbol is a
 * terminal by derivation. An empty production body is an ε-production.
 *
 * Text format ("A -> B c | ε"), one production group per line:
 *   S -> a S b
 *   S -> ε
 * The variable of the first line becomes the start symbol; "ε" denotes the
 * empty body, and so does a blank alternative — "a |", "| a" and "a | | b"
 * parse exactly like "a | ε | b" — while whitespace between symbols stays a
 * plain separator. Alternatives are separated with "|".
 *
 * Membership (`generate`) assumes single-character terminals and runs CYK
 * over an internal Chomsky normal form; `generate_tokens` accepts explicit
 * token slices instead. */
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

/* Error produced while parsing a grammar description. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /* One-based line number of the offending line (0 when there was no
     * production at all). */
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (line {})", self.message, self.line)
    }
}

impl std::error::Error for ParseError {}

/* Structure that represents a context-free grammar. */
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Grammar {
    nonterminals: BTreeSet<String>,
    productions: BTreeMap<String, Vec<Vec<String>>>,
    start_symbol: String,
}

/* Production table shared between the model and the transformation helpers. */
type ProductionTable = BTreeMap<String, Vec<Vec<String>>>;

impl Grammar {
    /* Creates a grammar whose start symbol is registered as a variable. */
    pub fn new(start_symbol: &str) -> Self {
        let mut grammar = Grammar::default();
        grammar.add_variable(start_symbol);
        grammar.start_symbol = start_symbol.to_string();
        grammar
    }

    pub fn start_symbol(&self) -> &str {
        &self.start_symbol
    }

    pub fn nonterminals(&self) -> &BTreeSet<String> {
        &self.nonterminals
    }

    /* Terminals are derived: every symbol occurring in a production body
     * without being a variable. */
    pub fn terminals(&self) -> BTreeSet<String> {
        let mut terminals = BTreeSet::new();
        for bodies in self.productions.values() {
            for body in bodies {
                for symbol in body {
                    if !self.nonterminals.contains(symbol) {
                        terminals.insert(symbol.clone());
                    }
                }
            }
        }
        terminals
    }

    pub fn productions_of(&self, variable: &str) -> Option<&Vec<Vec<String>>> {
        self.productions.get(variable)
    }

    /* All (variable, body) pairs sorted by variable name. */
    pub fn productions(&self) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for (variable, bodies) in &self.productions {
            for body in bodies {
                result.push((variable.clone(), body.clone()));
            }
        }
        result
    }

    pub fn add_variable(&mut self, name: &str) {
        self.nonterminals.insert(name.to_string());
        if self.start_symbol.is_empty() {
            self.start_symbol = name.to_string();
        }
    }

    /* Adds one alternative. An empty body slice is an ε-production. */
    pub fn add_production(&mut self, variable: &str, body: &[&str]) {
        self.add_variable(variable);
        let body: Vec<String> = body.iter().map(|s| s.to_string()).collect();
        self.productions
            .entry(variable.to_string())
            .or_default()
            .push(body);
    }

    /* True when the start symbol derives the empty word. */
    pub fn contains_epsilon(&self) -> bool {
        nullable_from_table(&self.productions).contains(&self.start_symbol)
    }

    /* Leftmost/rightmost derivations as the sequence of sentential forms,
     * starting at [start] and ending with the input string itself; None when
     * the input cannot be derived within `max_steps` rule applications.
     * Terminal prefixes/suffixes already produced never change, which prunes
     * the search against the input. Assumes single-character terminals so
     * sentential forms can be compared with the plain string. */
    pub fn derive_leftmost(&self, input: &str, max_steps: usize) -> Option<Vec<Vec<String>>> {
        self.derive_extreme(input.to_string(), true, max_steps)
    }

    pub fn derive_rightmost(&self, input: &str, max_steps: usize) -> Option<Vec<Vec<String>>> {
        self.derive_extreme(input.to_string(), false, max_steps)
    }

    fn derive_extreme(
        &self,
        input: String,
        leftmost: bool,
        max_steps: usize,
    ) -> Option<Vec<Vec<String>>> {
        // Best-first expansion ordered by variable count (then discovery
        // order): derivations shrink toward their final terminal form, so
        // this biases the search toward short completions without giving up
        // completeness — every ordering still visits all reachable forms
        // given enough budget. Breadth-ish ordering also avoids deep
        // recursion on grammars like S -> SS whose sentential forms wander
        // widely.
        use std::cmp::Reverse;
        #[derive(Clone)]
        struct Node {
            parent: Option<usize>,
            form: Vec<String>,
        }

        let initial = Node { parent: None, form: vec![self.start_symbol.clone()] };
        let var_count = |form: &[String]| {
            form.iter()
                .filter(|symbol| self.nonterminals.contains(*symbol))
                .count()
        };
        let mut nodes: Vec<Node> = vec![initial];
        let mut seen: HashSet<String> = HashSet::from([form_key(&nodes[0].form)]);
        // Min-heap over (variable count, discovery index).
        let mut heap: std::collections::BinaryHeap<Reverse<(usize, usize)>> =
            std::collections::BinaryHeap::from([Reverse((1, 0))]);
        let mut steps_left = max_steps;

        while let Some(Reverse((_, index))) = heap.pop() {
            let form = nodes[index].form.clone();
            if is_terminal_form(self, &form) {
                // A finished sentential form either spells the input or is a
                // dead end for this word only.
                if concatenate(&form) == input {
                    // Reconstruct the chain from parent links.
                    let mut chain_rev = Vec::new();
                    let mut cursor = Some(index);
                    while let Some(node_index) = cursor {
                        chain_rev.push(nodes[node_index].form.clone());
                        cursor = nodes[node_index].parent;
                    }
                    chain_rev.reverse();
                    return Some(chain_rev);
                }
                continue;
            }
            let position = match extreme_variable_position(self, &form, leftmost) {
                Some(found) => found,
                None => continue,
            };
            let variable = form[position].clone();
            let mut bodies: Vec<Vec<String>> = self
                .productions_of(&variable)
                .cloned()
                .unwrap_or_default();
            bodies.sort();
            for body in bodies {
                if steps_left == 0 {
                    break;
                }
                steps_left -= 1;

                let mut next = form.clone();
                next.splice(position..position + 1, body.iter().cloned());

                // Locked-terminal pruning.
                let terminal_matches_target = if leftmost {
                    let cut = extreme_variable_position(self, &next, true)
                        .unwrap_or(next.len());
                    input.starts_with(&concatenate(&next[..cut]))
                } else {
                    let cut = extreme_variable_position(self, &next, false)
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    input.ends_with(&concatenate(&next[cut..]))
                };
                if !terminal_matches_target {
                    continue;
                }
                let key = form_key(&next);
                if !seen.insert(key) {
                    continue;
                }
                let pushed_var_count = var_count(&next);
                nodes.push(Node { parent: Some(index), form: next });
                heap.push(Reverse((pushed_var_count, nodes.len() - 1)));
            }
        }
        None
    }

    /* Equivalent grammar in Chomsky normal form: every body is a single
     * terminal, two variables, or ε on the (fresh) start symbol. The output
     * keeps the exact language, ε included, via fresh helper variables. */
    pub fn to_chomsky_normal_form(&self) -> Grammar {
        let mut productions = self.productions.clone();
        let mut counter = 0usize;
        let mut names_in_use = all_symbol_names(&productions, &self.nonterminals);

        // Whether the final grammar must carry an ε-production.
        let has_epsilon = nullable_from_table(&productions).contains(&self.start_symbol);

        // 1. Fresh start symbol pointing at the original one.
        let start = fresh_name("S0", &names_in_use, &mut counter);
        names_in_use.insert(start.clone());
        productions
            .entry(start.clone())
            .or_default()
            .push(vec![self.start_symbol.clone()]);

        // 2. ε-elimination: rebuild every body dropping any subset of its
        // nullable positions, removing all empty bodies.
        let nullable = nullable_from_table(&productions);
        let mut stripped: ProductionTable = BTreeMap::new();
        for (variable, bodies) in &productions {
            let entry = stripped.entry(variable.clone()).or_default();
            for body in bodies {
                for variant in nullable_dropping_variants(body, &nullable) {
                    if !variant.is_empty() {
                        entry.push(variant);
                    }
                }
            }
        }
        let productions = dedup_bodies(stripped);

        // 3. Unit-pair elimination.
        let productions = eliminate_unit_productions(productions);

        // 4. Useless-symbol elimination.
        let mut productions = remove_useless_variables(productions, &start);

        // 5. Restore ε on the fresh start when the language contains it. This
        // happens after pruning so ε alone cannot resurrect dead variables;
        // the fresh start either inherited real bodies in step 3 or stands
        // alone carrying the empty word.
        if has_epsilon {
            productions
                .entry(start.clone())
                .or_default()
                .push(Vec::new());
        }

        // 6. Binarize bodies longer than two symbols.
        let productions = binarize_all(productions, &mut names_in_use, &mut counter);

        // 7. Isolate terminals inside length-2 bodies.
        let productions = isolate_terminals(productions, &mut names_in_use, &mut counter);

        let mut cnf = Grammar::default();
        cnf.productions = dedup_bodies(productions);
        cnf.start_symbol = start;
        for variable in cnf.productions.keys() {
            cnf.nonterminals.insert(variable.clone());
        }
        cnf
    }

    /* Membership assuming single-character terminals: CYK over an internal
     * Chomsky normal form. */
    pub fn generate(&self, input: &str) -> bool {
        let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
        self.generate_tokens(&tokens)
    }

    /* CYK membership over explicit tokens, compiled to CNF internally. */
    pub fn generate_tokens(&self, tokens: &[String]) -> bool {
        let cnf = self.to_chomsky_normal_form();
        cyk_accepts(&cnf, tokens)
    }
}

// ---------------------------------------------------------------------------
// Text parsing

/* Parses the "S -> a S b | ε" line format described in the module docs. */
pub fn parse_grammar(source: &str) -> Result<Grammar, ParseError> {
    let mut grammar = Grammar::default();
    let mut first_lhs: Option<String> = None;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (lhs, rhs) = match line.split_once("->") {
            Some(parts) => parts,
            None => {
                return Err(ParseError {
                    line: line_number,
                    message: format!("missing '->' in {:?}", line),
                })
            },
        };
        let lhs = lhs.trim();
        if lhs.is_empty() || lhs.contains(char::is_whitespace) {
            return Err(ParseError {
                line: line_number,
                message: "left-hand side must be a single symbol".to_string(),
            });
        }
        if first_lhs.is_none() {
            first_lhs = Some(lhs.to_string());
            grammar.add_variable(lhs);
        }
        for alternative in rhs.split('|') {
            let alternative = alternative.trim();
            // Lenient empty-body spellings: the literal ε or a blank
            // alternative (trailing/leading/doubled "|", empty right-hand
            // side).
            if alternative == "ε" || alternative.is_empty() {
                grammar.add_production(lhs, &[]);
            } else {
                let body: Vec<&str> = alternative.split_whitespace().collect();
                grammar.add_production(lhs, &body);
            }
        }
    }

    if let Some(start) = first_lhs {
        grammar.start_symbol = start;
    } else {
        return Err(ParseError {
            line: 0,
            message: "no productions found".to_string(),
        });
    }
    Ok(grammar)
}

// ---------------------------------------------------------------------------
// Derived membership machinery

type NullableSet = BTreeSet<String>;

/* Least fixpoint of variables that can derive the empty word. Bodies with no
 * entry in the table count as terminals and block nullability. */
fn nullable_from_table(productions: &ProductionTable) -> NullableSet {
    let mut nullable: NullableSet = BTreeSet::new();
    loop {
        let mut changed = false;
        for (variable, bodies) in productions {
            if nullable.contains(variable) {
                continue;
            }
            if bodies
                .iter()
                .any(|body| body.iter().all(|symbol| nullable.contains(symbol)))
            {
                nullable.insert(variable.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    nullable
}

fn form_key(form: &[String]) -> String {
    form.join("\u{1}")
}

fn is_terminal_form(grammar: &Grammar, form: &[String]) -> bool {
    form.iter().all(|symbol| !grammar.nonterminals.contains(symbol))
}

fn concatenate(symbols: &[String]) -> String {
    symbols.concat()
}

/* Position of the leftmost (leftmost=true) or rightmost variable. */
fn extreme_variable_position(
    grammar: &Grammar,
    form: &[String],
    leftmost: bool,
) -> Option<usize> {
    let indices: Box<dyn Iterator<Item = usize>> = if leftmost {
        Box::new(0..form.len())
    } else {
        Box::new((0..form.len()).rev())
    };
    for index in indices {
        if grammar.nonterminals.contains(&form[index]) {
            return Some(index);
        }
    }
    None
}


// ---------------------------------------------------------------------------
// CNF helpers

fn all_symbol_names(productions: &ProductionTable, declared: &BTreeSet<String>) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = declared.clone();
    for (variable, bodies) in productions {
        names.insert(variable.clone());
        for body in bodies {
            names.extend(body.iter().cloned());
        }
    }
    names
}

fn fresh_name(prefix: &str, taken: &BTreeSet<String>, counter: &mut usize) -> String {
    loop {
        let candidate = format!("{}{}", prefix, counter);
        *counter += 1;
        if !taken.contains(&candidate) {
            return candidate;
        }
    }
}

fn dedup_bodies(mut table: ProductionTable) -> ProductionTable {
    for bodies in table.values_mut() {
        bodies.sort();
        bodies.dedup();
    }
    table
}

/* Every nonempty variant of the body obtainable by omitting any subset of
 * its nullable positions (including omitting none). */
fn nullable_dropping_variants(body: &[String], nullable: &NullableSet) -> Vec<Vec<String>> {
    let droppable: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, symbol)| nullable.contains(*symbol))
        .map(|(index, _)| index)
        .collect();
    let variants_cap = 1u64 << droppable.len().min(20);
    let mut out = Vec::new();
    for mask in 0..variants_cap {
        let dropped: HashSet<usize> = droppable
            .iter()
            .enumerate()
            .filter(|(bit, _)| mask & (1u64 << bit) != 0)
            .map(|(_, &position)| position)
            .collect();
        let variant: Vec<String> = body
            .iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, symbol)| symbol.clone())
            .collect();
        out.push(variant);
    }
    out
}

/* Replaces unit chains A -> B by copying every non-unit body of B into A,
 * transitively; cycles collapse through the transitive closure. */
fn eliminate_unit_productions(productions: ProductionTable) -> ProductionTable {
    // Adjacency of unit edges.
    let mut unit_edges: HashMap<String, BTreeSet<String>> = HashMap::new();
    for (variable, bodies) in &productions {
        for body in bodies {
            if body.len() == 1 && productions.contains_key(&body[0]) {
                unit_edges
                    .entry(variable.clone())
                    .or_default()
                    .insert(body[0].clone());
            }
        }
    }

    // Transitive closure.
    let mut closure: HashMap<String, BTreeSet<String>> = unit_edges.clone();
    loop {
        let mut changed = false;
        let mut additions: Vec<(String, BTreeSet<String>)> = Vec::new();
        for (from, targets) in closure.iter() {
            let mut discovered: BTreeSet<String> = BTreeSet::new();
            for intermediate in targets {
                if let Some(via_targets) = closure.get(intermediate) {
                    for reached in via_targets {
                        if !targets.contains(reached) {
                            discovered.insert(reached.clone());
                        }
                    }
                }
            }
            if !discovered.is_empty() {
                additions.push((from.clone(), discovered));
            }
        }
        for (from, discovered) in additions {
            let entry = closure.get_mut(&from).expect("source key exists");
            for symbol in discovered {
                entry.insert(symbol);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut expanded: ProductionTable = BTreeMap::new();
    for (variable, bodies) in &productions {
        let entry = expanded.entry(variable.clone()).or_default();
        for body in bodies {
            let is_unit = body.len() == 1 && productions.contains_key(&body[0]);
            if !is_unit {
                entry.push(body.clone());
            }
        }
        if let Some(targets) = closure.get(variable) {
            for target in targets {
                if let Some(target_bodies) = productions.get(target) {
                    for body in target_bodies {
                        let target_body_is_unit =
                            body.len() == 1 && productions.contains_key(&body[0]);
                        if !target_body_is_unit {
                            entry.push(body.clone());
                        }
                    }
                }
            }
        }
    }
    dedup_bodies(expanded)
}

/* Removes non-generating variables first, then unreachable ones. The start
 * symbol is treated as the root; entries whose bodies die drop with them. */
fn remove_useless_variables(mut productions: ProductionTable, start: &str) -> ProductionTable {
    // Generating fixpoint.
    let mut generating: HashSet<String> = HashSet::new();
    loop {
        let mut changed = false;
        for (variable, bodies) in &productions {
            if generating.contains(variable) {
                continue;
            }
            if bodies.iter().any(|body| {
                body.iter().all(|symbol| {
                    !productions.contains_key(symbol) || generating.contains(symbol)
                })
            }) {
                generating.insert(variable.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    productions.retain(|variable, _| generating.contains(variable));
    let surviving: HashSet<String> = productions.keys().cloned().collect();
    for bodies in productions.values_mut() {
        bodies.retain(|body| {
            body.iter().all(|symbol| {
                !surviving.contains(symbol) || generating.contains(symbol)
            })
        });
    }
    productions.retain(|_, bodies| !bodies.is_empty());

    // Reachability from the start; if the start vanished (non-generating),
    // whatever remains is dropped entirely.
    let mut reachable: HashSet<String> = HashSet::new();
    if productions.contains_key(start) {
        let mut stack = vec![start.to_string()];
        while let Some(current) = stack.pop() {
            if !reachable.insert(current.clone()) {
                continue;
            }
            if let Some(bodies) = productions.get(&current) {
                for body in bodies {
                    for symbol in body {
                        if productions.contains_key(symbol) && !reachable.contains(symbol) {
                            stack.push(symbol.clone());
                        }
                    }
                }
            }
        }
    }
    productions.retain(|variable, _| reachable.contains(variable));
    let reachable_keys: HashSet<String> = productions.keys().cloned().collect();
    for bodies in productions.values_mut() {
        bodies.retain(|body| {
            body.iter().all(|symbol| {
                !reachable_keys.contains(symbol) || reachable.contains(symbol)
            })
        });
    }
    productions.retain(|_, bodies| !bodies.is_empty());
    productions
}

/* While any body longer than two symbols exists anywhere, lift its last two
 * symbols under a fresh helper variable. */
fn binarize_all(
    mut productions: ProductionTable,
    names_in_use: &mut BTreeSet<String>,
    counter: &mut usize,
) -> ProductionTable {
    loop {
        let victim = productions.iter().find_map(|(variable, bodies)| {
            bodies
                .iter()
                .find(|body| body.len() > 2)
                .map(|body| (variable.clone(), body.clone()))
        });
        let (variable, body) = match victim {
            Some(found) => found,
            None => break,
        };
        let helper = fresh_name("N", names_in_use, counter);
        names_in_use.insert(helper.clone());
        let tail: Vec<String> = body[body.len() - 2..].to_vec();
        let head: Vec<String> = body[..body.len() - 2].to_vec();
        let rewritten = head.into_iter().chain(std::iter::once(helper.clone())).collect();
        let bodies = productions.get_mut(&variable).expect("source key exists");
        bodies.retain(|candidate| *candidate != body);
        bodies.push(rewritten);
        productions.insert(helper, vec![tail]);
    }
    productions
}

/* Substitutes terminals inside length-2 bodies by fresh per-terminal helper
 * variables with single-terminal bodies. */
fn isolate_terminals(
    mut productions: ProductionTable,
    names_in_use: &mut BTreeSet<String>,
    counter: &mut usize,
) -> ProductionTable {
    let mut terminal_helpers: HashMap<String, String> = HashMap::new();
    let mut substitutions: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
    for (variable, bodies) in &productions {
        for body in bodies {
            if body.len() == 2
                && body.iter().any(|symbol| !productions.contains_key(symbol))
            {
                let replaced: Vec<String> = body
                    .iter()
                    .map(|symbol| {
                        if productions.contains_key(symbol) {
                            symbol.clone()
                        } else if let Some(existing) = terminal_helpers.get(symbol) {
                            existing.clone()
                        } else {
                            let helper = fresh_name("T", names_in_use, counter);
                            terminal_helpers.insert(symbol.clone(), helper.clone());
                            helper
                        }
                    })
                    .collect();
                substitutions.push((variable.clone(), body.clone(), replaced));
            }
        }
    }
    for (variable, old_body, new_body) in substitutions {
        if let Some(bodies) = productions.get_mut(&variable) {
            bodies.retain(|candidate| *candidate != old_body);
            bodies.push(new_body);
        }
    }
    for (terminal, helper) in terminal_helpers {
        productions.insert(helper, vec![vec![terminal]]);
    }
    productions
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Conversions to and from other machine families

use crate::finite_automata::FiniteAutomata;
use crate::pushdown_automata::PushdownAutomata;
use crate::state_machine::StateMachine;

impl Grammar {
    /* Converts a right-linear grammar into a nondeterministic finite
     * automaton over the same language. Production shapes allowed:
     * `A -> ε`, `A -> a` and `A -> a B` (terminal first, variable last);
     * unit productions `A -> B` are eliminated automatically before the
     * construction, anything else is rejected with a message naming the
     * production.
     * Construction states: one per variable plus one extra accepting state
     * that collects every terminal-only production. */
    pub fn to_finite_automata(&self) -> Result<FiniteAutomata, String> {
        let mut productions: ProductionTable = self.productions.clone();
        // A -> ε stops nullability; strip it after remembering which units it
        // feeds, mirroring the CNF pipeline's ε handling.
        let nullable = nullable_from_table(&productions);
        let mut stripped: ProductionTable = BTreeMap::new();
        for (variable, bodies) in &productions {
            let entry = stripped.entry(variable.clone()).or_default();
            for body in bodies {
                for variant in nullable_dropping_variants(body, &nullable) {
                    if !variant.is_empty() {
                        entry.push(variant);
                    }
                }
            }
        }
        productions = eliminate_unit_productions(dedup_bodies(stripped));

        for (variable, bodies) in &productions {
            for body in bodies {
                let well_shaped = match body.len() {
                    0 => false,
                    1 => !self.nonterminals.contains(&body[0]),
                    2 => !self.nonterminals.contains(&body[0]) && self.nonterminals.contains(&body[1]),
                    _ => false,
                };
                if !well_shaped {
                    return Err(format!(
                        "production {:?} -> {:?} is not right-linear (expected \"a\" or \"a B\" after unit elimination)",
                        variable, body
                    ));
                }
            }
        }

        let variables: Vec<String> = self.nonterminals.iter().cloned().collect();
        let index_of = |name: &str| -> usize {
            variables.iter().position(|v| v == name).expect("variable registered")
        };
        let accept_state = variables.len();

        let mut automata = FiniteAutomata::new();
        automata.add_n_states((variables.len() + 1) as u64);
        automata.make_initial(index_of(self.start_symbol()) as u64);
        automata.make_final(accept_state as u64);

        // ε-production information survives stripping through finality of the
        // variable's own state.
        for variable in &variables {
            if nullable.contains(variable) {
                automata.make_final(index_of(variable) as u64);
            }
        }
        for (variable, bodies) in &productions {
            for body in bodies {
                let from = index_of(variable) as u64;
                match body.len() {
                    1 => automata.add_transition(from, accept_state as u64, body[0].clone()),
                    2 => automata.add_transition(
                        from,
                        index_of(&body[1]) as u64,
                        body[0].clone(),
                    ),
                    _ => unreachable!("shape validated above"),
                }
            }
        }
        Ok(automata)
    }

    /* Builds a right-linear grammar generating exactly the language of the
     * automaton; variable names are chosen fresh so they never collide with
     * any transition label used as a terminal. Requires an initial state. */
    pub fn from_finite_automata(machine: &FiniteAutomata) -> Result<Grammar, String> {
        let initial_id = match machine.get_initial_state_id() {
            Some(id) => *id,
            None => return Err("The automaton has no initial state.".to_string()),
        };

        let labels_in_use: HashSet<String> = machine
            .get_states_by_id_ref()
            .values()
            .flat_map(|state| state.iter_by_transition())
            .flat_map(|(_, labels)| labels.iter().cloned())
            .collect();

        let mut name_by_id: HashMap<u64, String> = HashMap::new();
        let mut taken: HashSet<String> = HashSet::new();
        let candidates = "SABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut fallback_counter = 0usize;

        let mut assign_name = |id: u64| -> String {
            if let Some(existing) = name_by_id.get(&id) {
                return existing.clone();
            }
            let mut candidate = None;
            for c in candidates.chars() {
                let proposal = c.to_string();
                if !labels_in_use.contains(&proposal) && !taken.contains(&proposal) {
                    candidate = Some(proposal);
                    break;
                }
            }
            let name = match candidate {
                Some(found) => found,
                None => loop {
                    let proposal = format!("V{}", fallback_counter);
                    fallback_counter += 1;
                    if !labels_in_use.contains(&proposal) && !taken.contains(&proposal) {
                        break proposal;
                    }
                },
            };
            taken.insert(name.clone());
            name_by_id.insert(id, name.clone());
            name
        };

        // Deterministic order: sorted by state id.
        let mut ids: Vec<u64> = machine.get_states_by_id_ref().keys().copied().collect();
        ids.sort();
        for id in &ids {
            assign_name(*id);
        }

        let finals = machine.get_final_states();
        let start_variable = name_by_id[&initial_id].clone();
        let mut grammar = Grammar::new(&start_variable);
        // Every state becomes a declared variable even when it has no
        // outgoing productions (e.g. pure accepting states), otherwise its
        // occurrences would count as terminals.
        for name in name_by_id.values() {
            grammar.add_variable(name);
        }

        if finals.contains(&initial_id) {
            grammar.add_production(&start_variable, &[]);
        }
        for id in &ids {
            if let Some(state) = machine.get_states_by_id_ref().get(id) {
                for (target, labels) in state.iter_by_transition() {
                    for label in labels {
                        grammar.add_production(
                            &name_by_id[id],
                            &[label.as_str(), &name_by_id[target]],
                        );
                        if finals.contains(target) {
                            grammar.add_production(&name_by_id[id], &[label.as_str()]);
                        }
                    }
                }
            }
        }
        Ok(grammar)
    }

    /* Constructs the standard top-down recognizer pushdown automaton for the
     * grammar: expansion rules pop the leftmost stack entry (a variable) and
     * push the body's symbols as atomic entries, terminal rules match-and-pop
     * their token from the input, and reaching the bottom marker accepts (by
     * final state).
     *
     * Terminals may be multi-character tokens; variables keep their original
     * names because pushes are comma-segmented atomic entries. A trailing
     * ",ε" segment (a no-op) is appended to non-empty bodies so even
     * single-symbol bodies travel through the atomic path. Symbol names may
     * not contain the reserved characters ',', ';' or '/', nor be "ε"; the
     * bottom marker is chosen fresh and disjoint from every symbol. */
    pub fn to_pushdown_automata(&self) -> Result<PushdownAutomata, String> {
        let terminals = self.terminals();
        for symbol in self.nonterminals.iter().chain(terminals.iter()) {
            if symbol.contains(',')
                || symbol.contains(';')
                || symbol.contains('/')
                || symbol == "ε"
            {
                return Err(format!(
                    "symbol {:?} cannot be converted: ',', ';' and '/' are reserved by the pushdown label format",
                    symbol
                ));
            }
        }

        // Fresh bottom-of-stack marker, disjoint from every grammar symbol.
        let terminals = self.terminals();
        let symbols: HashSet<&String> = self
            .nonterminals
            .iter()
            .chain(terminals.iter())
            .collect();
        let mut marker = "Z".to_string();
        let mut marker_counter = 0usize;
        while symbols.contains(&marker) {
            marker = format!("Z{}", marker_counter);
            marker_counter += 1;
        }

        let mut pda = PushdownAutomata::new(marker.clone());
        pda.add_n_states(3);
        pda.make_initial(0);
        pda.make_final(2);

        // Bootstrap: push the start symbol over the bottom marker.
        pda.add_transition(
            0,
            1,
            format!("ε;{}/{},{}", marker, self.start_symbol, marker),
        );

        // Expansions: pop the variable, push the body's symbols as atomic
        // entries (leftmost on top). An empty body pushes "ε" (no-op after
        // the pop). The trailing ",ε" forces the atomic path even for
        // single-symbol bodies.
        for (variable, bodies) in &self.productions {
            for body in bodies {
                let pushed = if body.is_empty() {
                    "ε".to_string()
                } else {
                    format!("{},ε", body.join(","))
                };
                pda.add_transition(1, 1, format!("ε;{}/{}", variable, pushed));
            }
        }

        // Terminal matching consumes the whole token from the input.
        for terminal in &terminals {
            pda.add_transition(1, 1, format!("{};{}/ε", terminal, terminal));
        }

        // Bottom-of-stack marker reached: accept.
        pda.add_transition(1, 2, format!("ε;{}/{}", marker, marker));

        Ok(pda)
    }
}

/* Renders the grammar back into the text format accepted by parse_grammar,
 * variables grouped one line each in sorted order. */
impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for variable in &self.nonterminals {
            let bodies = match self.productions_of(variable) {
                Some(bodies) => bodies,
                None => continue,
            };
            write!(f, "{} ->", variable)?;
            for (index, body) in bodies.iter().enumerate() {
                if index > 0 {
                    write!(f, " |")?;
                }
                if body.is_empty() {
                    write!(f, " ε")?;
                } else {
                    write!(f, " {}", body.join(" "))?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// CYK over the CNF grammar

fn cyk_accepts(cnf: &Grammar, tokens: &[String]) -> bool {
    let n = tokens.len();
    if n == 0 {
        return cnf
            .productions_of(cnf.start_symbol())
            .map(|bodies| bodies.iter().any(Vec::is_empty))
            .unwrap_or(false);
    }
    // table[i][len] holds the variables deriving tokens[i .. i+len].
    let mut table: Vec<Vec<BTreeSet<String>>> = vec![vec![BTreeSet::new(); n + 1]; n];
    for i in 0..n {
        for (variable, bodies) in &cnf.productions {
            if bodies.iter().any(|body| body.len() == 1 && body[0] == tokens[i]) {
                table[i][1].insert(variable.clone());
            }
        }
    }
    for len in 2..=n {
        for i in 0..=(n - len) {
            for split in 1..len {
                if table[i][split].is_empty() || table[i + split][len - split].is_empty() {
                    continue;
                }
                // Borrow the two child sets and collect the variables this
                // split derives; inserting afterwards keeps the borrows and
                // the mutation from overlapping (a set union is
                // order-independent, so the result is identical).
                let mut to_add: Vec<String> = Vec::new();
                {
                    let left_set = &table[i][split];
                    let right_set = &table[i + split][len - split];
                    for (variable, bodies) in &cnf.productions {
                        if table[i][len].contains(variable) || to_add.contains(variable) {
                            continue;
                        }
                        let derives = bodies.iter().any(|body| {
                            body.len() == 2
                                && left_set.contains(&body[0])
                                && right_set.contains(&body[1])
                        });
                        if derives {
                            to_add.push(variable.clone());
                        }
                    }
                }
                for variable in to_add {
                    table[i][len].insert(variable);
                }
            }
        }
    }
    table[0][n].contains(cnf.start_symbol())
}
