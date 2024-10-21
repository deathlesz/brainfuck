use color_eyre::{
    eyre::{eyre, Context as _, ContextCompat},
    Result,
};
use inkwell::{
    attributes::Attribute,
    basic_block::BasicBlock,
    context::Context,
    passes::PassBuilderOptions,
    targets::{CodeModel, RelocMode, Target, TargetMachine, TargetTriple},
    OptimizationLevel,
};

use parser::Instruction;

use crate::{cli::Emit, ARGS};

#[derive(Debug, Clone)]
pub struct Compiler<const N: u64> {
    instructions: Vec<Instruction>,
}

impl<const N: u64> Compiler<N> {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn compile(self) -> Result<()> {
        let context = Context::create();
        let builder = context.create_builder();

        let module = context.create_module("brainfuck");

        let i8_type = context.i8_type();
        let i64_type = context.i64_type();

        let fn_type = i64_type.fn_type(&[], false);
        let main_fn = module.add_function("main", fn_type, None);

        let fn_type = i8_type.fn_type(&[], false);
        let getchar_fn =
            module.add_function("getchar", fn_type, Some(inkwell::module::Linkage::External));

        let fn_type = i8_type.fn_type(&[i8_type.into()], false);
        let putchar_fn =
            module.add_function("putchar", fn_type, Some(inkwell::module::Linkage::External));
        putchar_fn.add_attribute(
            inkwell::attributes::AttributeLoc::Param(0),
            context.create_enum_attribute(Attribute::get_named_enum_kind_id("noundef"), 0),
        );

        let entry_block = context.append_basic_block(main_fn, "entry");
        builder.position_at_end(entry_block);

        let array_type = i8_type.array_type(N as u32);
        let memory = builder.build_array_alloca(i8_type, i64_type.const_int(N, false), "memory")?;
        let memptr = builder.build_alloca(i64_type, "memptr")?;

        // initialize array with 0s
        builder.build_memset(
            memory,
            1,
            i8_type.const_zero(),
            i64_type.const_int(N, false),
        )?;
        builder.build_store(memptr, i64_type.const_zero())?;

        #[derive(Debug, Clone)]
        struct LoopBlock<'ctx> {
            start: BasicBlock<'ctx>,
            body: BasicBlock<'ctx>,
            end: BasicBlock<'ctx>,
        }

        let mut loop_stack = Vec::new();
        for instruction in self.instructions {
            use Instruction::*;

            match instruction {
                Add(n) => {
                    let memptr_value = builder
                        .build_load(i64_type, memptr, "add_idx")?
                        .into_int_value();
                    let elptr = unsafe {
                        builder.build_in_bounds_gep(
                            array_type,
                            memory,
                            &[i64_type.const_zero(), memptr_value],
                            "add_elptr",
                        )?
                    };

                    let value = builder
                        .build_load(i8_type, elptr, "add_value")?
                        .into_int_value();
                    let add =
                        builder.build_int_add(value, i8_type.const_int(n as u64, false), "add")?;
                    builder.build_store(elptr, add)?;
                }
                Move(n) if n != 0 => {
                    let memptr_value = builder
                        .build_load(i64_type, memptr, "move_idx")?
                        .into_int_value();
                    let add = builder.build_int_add(
                        memptr_value,
                        i64_type.const_int(n as u64, false),
                        "move_idx_add",
                    )?;

                    // Using urem/srem causes execution times to inflate...
                    // TODO: why?
                    //
                    // let result = builder.build_int_signed_rem(
                    //     add,
                    //     i64_type.const_int(N, false),
                    //     "move_idx_rem",
                    // )?;

                    builder.build_store(memptr, add)?;
                }
                In => {
                    let call = builder.build_call(getchar_fn, &[], "getchar")?;
                    let char = call.try_as_basic_value().left().unwrap(); // ?

                    let memptr_value = builder
                        .build_load(i64_type, memptr, "in_idx")?
                        .into_int_value();
                    let elptr = unsafe {
                        builder.build_in_bounds_gep(
                            array_type,
                            memory,
                            &[i64_type.const_zero(), memptr_value],
                            "in_elptr",
                        )?
                    };

                    builder.build_store(elptr, char)?;
                }
                Out => {
                    let memptr_value = builder
                        .build_load(i64_type, memptr, "out_idx")?
                        .into_int_value();
                    let elptr = unsafe {
                        builder.build_in_bounds_gep(
                            array_type,
                            memory,
                            &[i64_type.const_zero(), memptr_value],
                            "out_elptr",
                        )?
                    };

                    let value = builder
                        .build_load(i8_type, elptr, "out_value")?
                        .into_int_value();

                    builder.build_call(putchar_fn, &[value.into()], "putchar")?;
                }
                JumpIfZero(_) => {
                    let loop_block = LoopBlock {
                        start: context.append_basic_block(main_fn, "loop_start"),
                        body: context.append_basic_block(main_fn, "loop_body"),
                        end: context.append_basic_block(main_fn, "loop_end"),
                    };

                    loop_stack.push(loop_block.clone());

                    builder.build_unconditional_branch(loop_block.start)?;
                    builder.position_at_end(loop_block.start);

                    let memptr_value = builder
                        .build_load(i64_type, memptr, "loop_start_idx")?
                        .into_int_value();
                    let elptr = unsafe {
                        builder.build_in_bounds_gep(
                            array_type,
                            memory,
                            &[i64_type.const_zero(), memptr_value],
                            "loop_start_elptr",
                        )?
                    };

                    let value = builder
                        .build_load(i8_type, elptr, "loop_start_value")?
                        .into_int_value();
                    let cmp = builder.build_int_compare(
                        inkwell::IntPredicate::NE,
                        value,
                        i8_type.const_zero(),
                        "loop_start_cmp",
                    )?;

                    builder.build_conditional_branch(cmp, loop_block.body, loop_block.end)?;
                    builder.position_at_end(loop_block.body);
                }
                JumpIfNotZero(_) => {
                    let loop_block = loop_stack.pop().unwrap(); // will never fail
                    builder.build_unconditional_branch(loop_block.start)?;
                    builder.position_at_end(loop_block.end);
                }
                Clear => {
                    let memptr_value = builder
                        .build_load(i64_type, memptr, "clear_idx")?
                        .into_int_value();
                    let elptr = unsafe {
                        builder.build_in_bounds_gep(
                            array_type,
                            memory,
                            &[i64_type.const_zero(), memptr_value],
                            "clear_elptr",
                        )?
                    };

                    builder.build_store(elptr, i8_type.const_zero())?;
                }
                AddTo(n) => {
                    let memptr_value = builder
                        .build_load(i64_type, memptr, "add_to_idx")?
                        .into_int_value();

                    let add = builder.build_int_add(
                        memptr_value,
                        i64_type.const_int(n as u64, false),
                        "add_to_idx_add",
                    )?;
                    let rem = builder.build_int_unsigned_rem(
                        add,
                        i64_type.const_int(N, false),
                        "add_to_idx_rem",
                    )?;

                    let (elptr_current, elptr_to) = unsafe {
                        (
                            builder.build_in_bounds_gep(
                                array_type,
                                memory,
                                &[i64_type.const_zero(), memptr_value],
                                "add_to_elptr_current",
                            )?,
                            builder.build_in_bounds_gep(
                                array_type,
                                memory,
                                &[i64_type.const_zero(), rem],
                                "add_to_elptr_to",
                            )?,
                        )
                    };

                    let value_current = builder
                        .build_load(i8_type, elptr_current, "add_to_value_current")?
                        .into_int_value();
                    let value_to = builder
                        .build_load(i8_type, elptr_to, "add_to_value_to")?
                        .into_int_value();

                    let add = builder.build_int_add(value_to, value_current, "add_to_add")?;
                    builder.build_store(elptr_to, add)?;
                }
                _ => {}
            }
        }

        builder.build_return(Some(&i64_type.const_zero()))?;

        Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        let target_triple = match ARGS.target {
            Some(ref triple) => TargetTriple::create(triple),
            None => TargetMachine::get_default_triple(),
        };
        let cpu = match ARGS.target {
            Some(ref cpu) => cpu,
            None => &TargetMachine::get_host_cpu_name().to_string(),
        };
        let features = match ARGS.features {
            Some(ref features) if features == "native" => {
                &TargetMachine::get_host_cpu_features().to_string()
            }
            Some(ref features) => features,
            None => "+cmov,+cx8,+fxsr,+mmx,+sse,+sse2", // x86_64-v1 (baseline)
        };

        let target = Target::from_triple(&target_triple).map_err(|_| eyre!("bad target triple"))?;
        let target_machine = target
            .create_target_machine(
                &target_triple,
                cpu,
                features,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .wrap_err("failed to create target machine")?;

        module.set_triple(&target_triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        if ARGS.optimize {
            module.run_passes("annotation2metadata,forceattrs,inferattrs,function<eager-inv>(lower-expect,simplifycfg,early-cse),openmp-opt,ipsccp,called-value-propagation,globalopt,function<eager-inv>(instcombine<no-verify-fixpoint>,simplifycfg),always-inline,require<globals-aa>,function(invalidate<aa>),require<profile-summary>,cgscc(devirt<4>(inline,function-attrs,openmp-opt-cgscc,function<eager-inv;no-rerun>(early-cse<memssa>,speculative-execution,jump-threading,correlated-propagation,simplifycfg,instcombine<no-verify-fixpoint>,aggressive-instcombine,libcalls-shrinkwrap,tailcallelim,simplifycfg,reassociate,constraint-elimination,loop-mssa(loop-instsimplify,loop-simplifycfg,licm<no-allowspeculation>,loop-rotate<header-duplication;no-prepare-for-lto>,licm<allowspeculation>,simple-loop-unswitch<no-nontrivial;trivial>),simplifycfg,instcombine<no-verify-fixpoint>,loop(loop-idiom,indvars,loop-deletion,loop-unroll-full),vector-combine,mldst-motion<no-split-footer-bb>,sccp,bdce,instcombine<no-verify-fixpoint>,jump-threading,correlated-propagation,adce,memcpyopt,dse,move-auto-init,loop-mssa(licm<allowspeculation>),simplifycfg,instcombine<no-verify-fixpoint>),function-attrs,function(require<should-not-run-function-passes>))),deadargelim,globalopt,globaldce,elim-avail-extern,rpo-function-attrs,recompute-globalsaa,function<eager-inv>(lower-constant-intrinsics,loop(loop-rotate<header-duplication;no-prepare-for-lto>,loop-deletion),loop-distribute,inject-tli-mappings,loop-vectorize<no-interleave-forced-only;no-vectorize-forced-only>,infer-alignment,loop-load-elim,instcombine<no-verify-fixpoint>,simplifycfg,slp-vectorizer,vector-combine,instcombine<no-verify-fixpoint>,loop-unroll<O2>,transform-warning,loop-mssa(licm<allowspeculation>),infer-alignment,alignment-from-assumptions,loop-sink,instsimplify,div-rem-pairs,tailcallelim,simplifycfg),globaldce,constmerge,cg-profile,rel-lookup-table-converter,function(annotation-remarks),verify", &target_machine, PassBuilderOptions::create()).unwrap();
        }

        if ARGS.run {
            let engine = module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)
                .map_err(|_| eyre!("failed to create JIT execution engine"))?;

            unsafe {
                let func = engine
                    .get_function::<unsafe extern "C" fn() -> i64>("main")
                    .unwrap();

                func.call();
            }

            return Ok(());
        }

        let source = ARGS
            .source
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("live.bf"));
        let output = source.with_extension(ARGS.emit.extension());
        let output = ARGS.output.as_ref().unwrap_or(&output);

        match ARGS.emit {
            Emit::LLVMIr => std::fs::write(output, module.to_string())
                .wrap_err("failed to emit LLVM IR to output file")?,
            Emit::Assembly => target_machine
                .write_to_file(&module, inkwell::targets::FileType::Assembly, output)
                .map_err(|_| eyre!("failed to write to emit assembly to output file"))?,
            Emit::Object => target_machine
                .write_to_file(&module, inkwell::targets::FileType::Object, output)
                .map_err(|_| eyre!("failed to write to write to output file"))?,
        };

        Ok(())
    }
}
