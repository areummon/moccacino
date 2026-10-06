use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (line {})", self.message, self.line)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Grammar {
    nonterminals: BTreeSet<String>,
    productions: BTreeMap<String, Vec<Vec<String>>>,
    start_symbol: String,
}

type ProductionTable = BTreeMap<String, Vec<Vec<String>>>;

impl Grammar {
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

    pub fn add_production(&mut self, variable: &str, body: &[&str]) {
        self.add_variable(variable);
        let body: Vec<String> = body.iter().map(|s| s.to_string()).collect();
        self.productions
            .entry(variable.to_string())
            .or_default()
            .push(body);
    }

    fn production_table(&self) -> ProductionTable {
        let mut table = self.productions.clone();
        for variable in &self.nonterminals {
            table.entry(variable.clone()).or_default();
        }
        table
    }

    pub fn contains_epsilon(&self) -> bool {
        nullable_from_table(&self.productions).contains(&self.start_symbol)
    }

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
        let mut heap: std::collections::BinaryHeap<Reverse<(usize, usize)>> =
            std::collections::BinaryHeap::from([Reverse((1, 0))]);
        let nullable = nullable_from_table(&self.production_table());
        let input_length = input.chars().count();
        let minimum_length = |form: &[String]| -> usize {
            form.iter()
                .map(|symbol| {
                    if !self.nonterminals.contains(symbol) {
                        symbol.chars().count()
                    } else if nullable.contains(symbol) {
                        0
                    } else {
                        1
                    }
                })
                .sum()
        };
        let sorted_productions: HashMap<&String, Vec<&Vec<String>>> = self
            .productions
            .iter()
            .map(|(variable, bodies)| {
                let mut sorted: Vec<&Vec<String>> = bodies.iter().collect();
                sorted.sort();
                (variable, sorted)
            })
            .collect();
        let mut steps_left = max_steps;

        while let Some(Reverse((_, index))) = heap.pop() {
            let form = nodes[index].form.clone();
            if is_terminal_form(self, &form) {
                if form.concat() == input {
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
            for body in sorted_productions.get(&form[position]).map(Vec::as_slice).unwrap_or_default() {
                if steps_left == 0 {
                    break;
                }
                steps_left -= 1;

                let mut next = form.clone();
                next.splice(position..position + 1, body.iter().cloned());

                let terminal_matches_target = if leftmost {
                    let cut = extreme_variable_position(self, &next, true)
                        .unwrap_or(next.len());
                    input.starts_with(&next[..cut].concat())
                } else {
                    let cut = extreme_variable_position(self, &next, false)
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    input.ends_with(&next[cut..].concat())
                };
                if !terminal_matches_target || minimum_length(&next) > input_length {
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

    pub fn to_chomsky_normal_form(&self) -> Grammar {
        let mut productions = self.production_table();
        let mut counter = 0usize;
        let mut names_in_use = all_symbol_names(&productions, &self.nonterminals);

        let has_epsilon = nullable_from_table(&productions).contains(&self.start_symbol);

        let start = fresh_name("S0", &names_in_use, &mut counter);
        names_in_use.insert(start.clone());
        productions
            .entry(start.clone())
            .or_default()
            .push(vec![self.start_symbol.clone()]);

        let productions = without_nullable_bodies(&productions, &nullable_from_table(&productions));

        let productions = eliminate_unit_productions(productions);

        let mut productions = remove_useless_variables(productions, &start);

        if has_epsilon {
            productions
                .entry(start.clone())
                .or_default()
                .push(Vec::new());
        }

        let productions = binarize_all(productions, &mut names_in_use, &mut counter);

        let productions = isolate_terminals(productions, &mut names_in_use, &mut counter);

        let productions = dedup_bodies(productions);
        Grammar {
            nonterminals: productions.keys().cloned().collect(),
            productions,
            start_symbol: start,
        }
    }

    pub fn generate(&self, input: &str) -> bool {
        let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
        self.generate_tokens(&tokens)
    }

    pub fn generate_tokens(&self, tokens: &[String]) -> bool {
        let cnf = self.to_chomsky_normal_form();
        cyk_accepts(&cnf, tokens)
    }
}

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
                    message: format!("missing '->' in {line:?}"),
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
            let body: Vec<&str> = alternative
                .split_whitespace()
                .filter(|symbol| *symbol != "ε")
                .collect();
            grammar.add_production(lhs, &body);
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

type NullableSet = BTreeSet<String>;

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

fn extreme_variable_position(grammar: &Grammar, form: &[String], leftmost: bool) -> Option<usize> {
    let is_variable = |symbol: &String| grammar.nonterminals.contains(symbol);
    if leftmost {
        form.iter().position(is_variable)
    } else {
        form.iter().rposition(is_variable)
    }
}

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
        let candidate = format!("{prefix}{counter}");
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

fn without_nullable_bodies(productions: &ProductionTable, nullable: &NullableSet) -> ProductionTable {
    let stripped = productions
        .iter()
        .map(|(variable, bodies)| {
            let variants = bodies
                .iter()
                .flat_map(|body| nullable_dropping_variants(body, nullable))
                .filter(|variant| !variant.is_empty())
                .collect();
            (variable.clone(), variants)
        })
        .collect();
    dedup_bodies(stripped)
}

fn nullable_dropping_variants(body: &[String], nullable: &NullableSet) -> Vec<Vec<String>> {
    let droppable = body.iter().filter(|symbol| nullable.contains(*symbol)).count();
    (0..1u64 << droppable.min(20))
        .map(|mask| {
            let mut bit = 0;
            body.iter()
                .filter(|symbol| {
                    if !nullable.contains(*symbol) {
                        return true;
                    }
                    bit += 1;
                    bit > 20 || mask & (1 << (bit - 1)) == 0
                })
                .cloned()
                .collect()
        })
        .collect()
}

fn eliminate_unit_productions(productions: ProductionTable) -> ProductionTable {
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

fn remove_useless_variables(mut productions: ProductionTable, start: &str) -> ProductionTable {
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
    let variables: HashSet<String> = productions.keys().cloned().collect();
    productions.retain(|variable, _| generating.contains(variable));
    for bodies in productions.values_mut() {
        bodies.retain(|body| {
            body.iter().all(|symbol| {
                !variables.contains(symbol) || generating.contains(symbol)
            })
        });
    }
    productions.retain(|_, bodies| !bodies.is_empty());

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

use crate::finite_automata::FiniteAutomata;
use crate::pushdown_automata::PushdownAutomata;
use crate::state_machine::StateMachine;

impl Grammar {
    pub fn to_finite_automata(&self) -> Result<FiniteAutomata, String> {
        let table = self.production_table();
        let nullable = nullable_from_table(&table);
        let productions = eliminate_unit_productions(without_nullable_bodies(&table, &nullable));

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
                        "production {variable:?} -> {body:?} is not right-linear (expected \"a\" or \"a B\" after unit elimination)"
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
                    let proposal = format!("V{fallback_counter}");
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

        let mut ids: Vec<u64> = machine.get_states_by_id_ref().keys().copied().collect();
        ids.sort();
        for id in &ids {
            assign_name(*id);
        }

        let finals = machine.get_final_states();
        let start_variable = name_by_id[&initial_id].clone();
        let mut grammar = Grammar::new(&start_variable);
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
                        let read: Vec<&str> = if label.is_empty() || label == "ε" {
                            Vec::new()
                        } else {
                            vec![label.as_str()]
                        };
                        let mut body = read.clone();
                        body.push(&name_by_id[target]);
                        grammar.add_production(&name_by_id[id], &body);
                        if finals.contains(target) {
                            grammar.add_production(&name_by_id[id], &read);
                        }
                    }
                }
            }
        }
        grammar.drop_bodies_with_dead_variables();
        Ok(grammar)
    }

    fn drop_bodies_with_dead_variables(&mut self) {
        loop {
            let dead: HashSet<String> = self
                .nonterminals
                .iter()
                .filter(|variable| {
                    self.productions.get(*variable).is_none_or(|bodies| bodies.is_empty())
                })
                .cloned()
                .collect();
            let mut changed = false;
            for bodies in self.productions.values_mut() {
                let before = bodies.len();
                bodies.retain(|body| !body.iter().any(|symbol| dead.contains(symbol)));
                changed |= bodies.len() != before;
            }
            if !changed {
                break;
            }
        }
        for bodies in self.productions.values_mut() {
            bodies.sort();
            bodies.dedup();
        }
    }

    pub fn to_pushdown_automata(&self) -> Result<PushdownAutomata, String> {
        let terminals = self.terminals();
        for symbol in self.nonterminals.iter().chain(terminals.iter()) {
            if symbol.contains(',')
                || symbol.contains(';')
                || symbol.contains('/')
                || symbol == "ε"
            {
                return Err(format!(
                    "symbol {symbol:?} cannot be converted: ',', ';' and '/' are reserved by the pushdown label format"
                ));
            }
        }

        let symbols: HashSet<&String> = self
            .nonterminals
            .iter()
            .chain(terminals.iter())
            .collect();
        let mut marker = "Z".to_string();
        let mut marker_counter = 0usize;
        while symbols.contains(&marker) {
            marker = format!("Z{marker_counter}");
            marker_counter += 1;
        }

        let mut pda = PushdownAutomata::new(marker.clone());
        pda.add_n_states(3);
        pda.make_initial(0);
        pda.make_final(2);

        pda.add_transition(
            0,
            1,
            format!("ε;{}/{},{}", marker, self.start_symbol, marker),
        );

        for (variable, bodies) in &self.productions {
            for body in bodies {
                let pushed = if body.is_empty() {
                    "ε".to_string()
                } else {
                    format!("{},ε", body.join(","))
                };
                pda.add_transition(1, 1, format!("ε;{variable}/{pushed}"));
            }
        }

        for terminal in &terminals {
            pda.add_transition(1, 1, format!("{terminal};{terminal}/ε"));
        }

        pda.add_transition(1, 2, format!("ε;{marker}/{marker}"));

        Ok(pda)
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.nonterminals.get(&self.start_symbol);
        let others = self.nonterminals.iter().filter(|variable| **variable != self.start_symbol);
        for variable in start.into_iter().chain(others) {
            let bodies = match self.productions_of(variable) {
                Some(bodies) => bodies,
                None => continue,
            };
            write!(f, "{variable} ->")?;
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

fn cyk_accepts(cnf: &Grammar, tokens: &[String]) -> bool {
    let n = tokens.len();
    if n == 0 {
        return cnf
            .productions_of(cnf.start_symbol())
            .map(|bodies| bodies.iter().any(Vec::is_empty))
            .unwrap_or(false);
    }
    let rules = || {
        cnf.productions
            .iter()
            .flat_map(|(variable, bodies)| bodies.iter().map(move |body| (variable.as_str(), body.as_slice())))
    };
    let binary: Vec<(&str, &str, &str)> = rules()
        .filter_map(|(variable, body)| match body {
            [left, right] => Some((variable, left.as_str(), right.as_str())),
            _ => None,
        })
        .collect();
    let mut table: Vec<Vec<BTreeSet<&str>>> = vec![vec![BTreeSet::new(); n + 1]; n];
    for (i, token) in tokens.iter().enumerate() {
        table[i][1] = rules()
            .filter(|(_, body)| matches!(body, [symbol] if symbol == token))
            .map(|(variable, _)| variable)
            .collect();
    }
    for len in 2..=n {
        for i in 0..=(n - len) {
            let mut cell: BTreeSet<&str> = BTreeSet::new();
            for split in 1..len {
                let (left, right) = (&table[i][split], &table[i + split][len - split]);
                if left.is_empty() || right.is_empty() {
                    continue;
                }
                cell.extend(
                    binary
                        .iter()
                        .filter(|(_, b, c)| left.contains(b) && right.contains(c))
                        .map(|(a, _, _)| *a),
                );
            }
            table[i][len] = cell;
        }
    }
    table[0][n].contains(cnf.start_symbol())
}
