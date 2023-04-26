use std::{collections::HashMap, io::{self, Write}, fs::File};

use anyhow::Result;

use crate::{parser_generator::{TransitionMap, LALR1ProductionRuleData, TransitionData}, parser_generator_lr0, hashable_set::HashableSet};

pub fn export_closures(
    closure_map: &HashMap<HashableSet<LALR1ProductionRuleData>, String>,
    out_path: &str,
) -> Result<()> {
    let mut writer = io::BufWriter::new(File::create(out_path)?);

    let mut sorted_closure_map: Vec<(&HashableSet<LALR1ProductionRuleData>, &String)> = closure_map.iter().collect();
    sorted_closure_map
        .sort_by(|a, b| {
            let a_state = a.1[1..].parse::<usize>().unwrap();
            let b_state = b.1[1..].parse::<usize>().unwrap();

            a_state.cmp(&b_state)
        });
    for (closure, state) in sorted_closure_map {
        writeln!(writer, "{}:", state)?;

        for rule in closure.set.iter() {
            let mut rule_str = format!("{} -> ", rule.left);

            for (i, symbol) in rule.right.iter().enumerate() {
                if i == rule.dot {
                    rule_str.push('・');
                }

                rule_str.push_str(symbol);
                rule_str.push(' ');
            }

            if rule.dot == rule.right.len() {
                rule_str.push('・');
            }

            let follow_str = rule.follow.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ");

            writeln!(writer, "  - rule: \"{}\"", rule_str)?;
            writeln!(writer, "    follow: \"{}\"", follow_str)?;
        }
    }

    Ok(())
}

pub fn export_transitions(
    transition_map: &TransitionMap,
    closure_map: &HashMap<HashableSet<LALR1ProductionRuleData>, String>,
    parser_generator_lr0: &parser_generator_lr0::ParserGeneratorLR0,
    out_path: &str,
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().quote_style(csv::QuoteStyle::Necessary).from_path(out_path)?;

    writer.write_field("")?;

    let mut symbols: Vec<&str> = parser_generator_lr0.terminal_tokens.iter().map(|x| *x).collect();
    symbols.push(&"[END]");
    symbols.extend(parser_generator_lr0.non_terminal_tokens.clone());

    for symbol in &symbols {
        writer.write_field(symbol)?;
    }

    writer.write_record(None::<&[u8]>)?;

    for i in 0..closure_map.len() {
        let state = format!("I{}", i);
        writer.write_field(&state)?;

        for symbol in &symbols {
            match transition_map.transitions.get(&(state.clone(), symbol.to_string())) {
                Some(transition) => {
                    match transition {
                        TransitionData::Shift { target } => {
                            writer.write_field(&format!("s{}", &target[1..]))?;
                        }
                        TransitionData::Reduce { rule } => {
                            writer.write_field(&format!("r{}", rule.left))?;
                        }
                        TransitionData::Accept => {
                            writer.write_field("A")?;
                        }
                        TransitionData::ReduceReduceConflict { rules } => {
                            writer.write_field(rules.iter().map(|x| x.left.to_string()).collect::<Vec<String>>().join("/"))?;
                        }
                    }
                }
                None => {
                    writer.write_field("")?;
                }
            }
        }

        writer.write_record(None::<&[u8]>)?;
    }

    Ok(())
}