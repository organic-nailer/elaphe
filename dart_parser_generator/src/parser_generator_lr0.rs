use crate::{
    grammar::EPSILON,
    parser_generator::{LALR1ProductionRuleData, State, Token},
    hashable_set::HashableSet,
};
use std::{collections::{HashMap, HashSet}, fmt, hash::Hash};

pub struct ParserGeneratorLR0<'a> {
    pub rules: &'a Vec<ProductionRuleData>,
    pub start_symbol: Token,
    pub terminal_tokens: HashSet<Token>,
    pub non_terminal_tokens: HashSet<Token>,
    pub goto_table: HashMap<(State, Token), State>,
    pub closure_map: HashMap<HashableSet<LRProductionRuleData>, State>,
}

impl ParserGeneratorLR0<'_> {
    pub fn get_kernel_map(&self) -> HashMap<State, HashableSet<LRProductionRuleData>> {
        let mut kernel_map: HashMap<State, HashableSet<LRProductionRuleData>> = HashMap::new();

        for (closure_rules, closure_state) in &self.closure_map {
            kernel_map.insert(
                closure_state.to_string(),
                closure_rules.set.iter()
                    .filter(|d| d.dot != 0 || d.left == "[START]")
                    .cloned()
                    .collect(),
            );
        }

        kernel_map
    }
}

impl ParserGeneratorLR0<'_> {
    pub fn new<'a>(
        rules: &'a Vec<ProductionRuleData>,
        start_symbol: Token,
    ) -> ParserGeneratorLR0<'a> {
        let (terminal_tokens, non_terminal_tokens) = ParserGeneratorLR0::calc_token_kind(rules);
        let (goto_table, closure_map) =
            ParserGeneratorLR0::calc_goto(rules, start_symbol, &non_terminal_tokens);

        ParserGeneratorLR0 {
            rules,
            start_symbol,
            terminal_tokens: terminal_tokens.into_iter().collect(),
            non_terminal_tokens: non_terminal_tokens.into_iter().collect(),
            goto_table,
            closure_map,
        }
    }

    /// Calculate the kind of tokens
    /// Return (terminal_tokens, non_terminal_tokens)
    fn calc_token_kind(rules: &Vec<ProductionRuleData>) -> (HashSet<Token>, HashSet<Token>) {
        let mut terminal_tokens: HashSet<Token> = HashSet::new();
        let non_terminal_tokens: HashSet<Token> = rules.iter().map(|rule| rule.left).collect();

        for rule in rules.iter() {
            for token in rule.right.iter() {
                if non_terminal_tokens.contains(token) {
                    continue;
                }
                terminal_tokens.insert(token);
            }
        }
        terminal_tokens.remove(EPSILON);

        (terminal_tokens, non_terminal_tokens)
    }

    fn calc_goto(
        rules: &Vec<ProductionRuleData>,
        start_symbol: Token,
        non_terminal_tokens: &HashSet<Token>,
    ) -> (
        HashMap<(State, Token), State>,
        HashMap<HashableSet<LRProductionRuleData>, State>,
    ) {
        let mut goto_map: HashMap<(State, Token), State> = HashMap::new();

        let initial_grammar = ProductionRuleData {
            left: "[START]",
            right: vec![start_symbol],
        };
        let mut extended_rules = rules.clone();
        extended_rules.push(initial_grammar.clone());

        let mut closure_map: HashMap<HashableSet<LRProductionRuleData>, State> = HashMap::new();
        let mut closure_index = 0;
        let mut init_set = HashableSet::new();
        init_set.insert(initial_grammar.to_lr());
        closure_map.insert(
            ParserGeneratorLR0::get_closure(init_set, &extended_rules, non_terminal_tokens),
            format!("I{}", closure_index),
        );

        let mut updated = true;
        while updated {
            updated = false;
            for (closure_rules, closure_state) in closure_map.clone() {
                let transition_tokens = closure_rules
                    .iter()
                    .filter(|r| !r.reducible)
                    .map(|r| r.right[r.dot])
                    .collect::<HashSet<Token>>();

                for token in transition_tokens {
                    if goto_map.contains_key(&(closure_state.clone(), token)) {
                        continue;
                    }
                    updated = true;
                    let goto_set = ParserGeneratorLR0::get_goto(
                        &closure_rules,
                        &extended_rules,
                        non_terminal_tokens,
                        token,
                    );
                    if goto_set.len() != 0 {
                        match closure_map.get(&goto_set) {
                            Some(goto_name) => {
                                goto_map
                                    .insert((closure_state.clone(), token), goto_name.to_string());
                            }
                            None => {
                                closure_index += 1;
                                closure_map.insert(goto_set, format!("I{}", closure_index));
                                goto_map.insert(
                                    (closure_state.clone(), token),
                                    format!("I{}", closure_index),
                                );
                            }
                        }
                    }
                }
            }
        }

        (goto_map, closure_map)
    }

    fn get_closure(
        input: HashableSet<LRProductionRuleData>,
        grammar_rules: &Vec<ProductionRuleData>,
        non_terminal_tokens: &HashSet<Token>,
    ) -> HashableSet<LRProductionRuleData> {
        let mut result = input.clone();
        let mut added_tokens: HashSet<Token> = HashSet::new();
        let mut updated = true;
        while updated {
            updated = false;
            for rule in result.set.clone() {
                if rule.reducible {
                    continue;
                }
                let target = rule.right[rule.dot];
                if !added_tokens.contains(target) && non_terminal_tokens.contains(target) {
                    added_tokens.insert(target);
                    for r in grammar_rules.iter().filter(|r| r.left == target) {
                        updated = result.insert(r.to_lr()) || updated;
                    }
                }
            }
        }
        result
    }

    fn get_goto(
        input: &HashableSet<LRProductionRuleData>,
        grammar_rules: &Vec<ProductionRuleData>,
        non_terminal_tokens: &HashSet<Token>,
        token: Token,
    ) -> HashableSet<LRProductionRuleData> {
        ParserGeneratorLR0::get_closure(
            input
                .iter()
                .filter(|r| !r.reducible && r.right[r.dot] == token)
                .map(|r| r.shift().unwrap())
                .collect(),
            grammar_rules,
            non_terminal_tokens,
        )
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LRProductionRuleData {
    pub left: Token,
    pub right: Vec<Token>,
    pub dot: usize,
    pub reducible: bool,
}

impl fmt::Display for LRProductionRuleData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> ", self.left)?;
        for (i, token) in self.right.iter().enumerate() {
            if i == self.dot {
                write!(f, "・")?;
            }
            write!(f, "{} ", token)?;
        }
        if self.dot == self.right.len() {
            write!(f, "・")?;
        }
        Ok(())
    }
}

impl LRProductionRuleData {
    pub fn shift(&self) -> Result<LRProductionRuleData, String> {
        if self.reducible {
            return Err("Cannot shift a reducible rule.".to_string());
        }
        Ok(LRProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: self.dot + 1,
            reducible: self.dot + 1 >= self.right.len(),
        })
    }

    pub fn to_lalr1(&self, follow: HashableSet<Token>) -> LALR1ProductionRuleData {
        LALR1ProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: self.dot,
            reducible: self.reducible,
            follow,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProductionRuleData {
    pub left: Token,
    pub right: Vec<Token>,
}

impl fmt::Display for ProductionRuleData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> ", self.left)?;
        for token in self.right.iter() {
            write!(f, "{} ", token)?;
        }
        Ok(())
    }
}

impl ProductionRuleData {
    pub fn to_lr(&self) -> LRProductionRuleData {
        LRProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: 0,
            reducible: self.right.len() == 1 && self.right[0] == EPSILON,
        }
    }

    pub fn to_lalr1(&self, follow: HashableSet<Token>) -> LALR1ProductionRuleData {
        LALR1ProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: 0,
            reducible: self.right.len() == 1 && self.right[0] == EPSILON,
            follow,
        }
    }
}
