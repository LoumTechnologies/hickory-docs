/*
 * Remill TraceLifter C++ bridge for ELF → WASM transpilation.
 *
 * Wraps Remill's C++ API behind a C-compatible FFI for consumption by Rust.
 * This file is compiled by cmake as part of the build.rs process, only when
 * Remill is installed on the system.
 */

#include "remill_bridge.h"

#include <cstring>
#include <memory>
#include <string>
#include <vector>

#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Verifier.h>
#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>

#include <remill/Arch/Arch.h>
#include <remill/Arch/Name.h>
#include <remill/BC/IntrinsicTable.h>
#include <remill/BC/Lifter.h>
#include <remill/BC/Util.h>
#include <remill/OS/OS.h>

struct CodeRegion {
    std::string name;
    uint64_t vaddr;
    std::vector<uint8_t> bytes;
};

struct DataSectionInfo {
    std::string name;
    uint64_t vaddr;
    std::vector<uint8_t> bytes;
    bool writable;
};

struct RemillContextImpl {
    std::string semantics_dir;
    std::vector<CodeRegion> code_regions;
    std::vector<DataSectionInfo> data_sections;
    uint64_t entry_point = 0;
    std::string last_error;
};

struct RemillModuleImpl {
    std::unique_ptr<llvm::LLVMContext> llvm_ctx;
    std::unique_ptr<llvm::Module> module;
};

/* Custom memory provider for Remill's TraceManager. */
class Elf2WasmTraceManager : public remill::TraceManager {
public:
    explicit Elf2WasmTraceManager(const RemillContextImpl *ctx)
        : ctx_(ctx) {}

    /* Called by TraceLifter to read bytes at a given address. */
    bool TryReadExecutableByte(uint64_t addr, uint8_t *byte) override {
        for (const auto &region : ctx_->code_regions) {
            if (addr >= region.vaddr &&
                addr < region.vaddr + region.bytes.size()) {
                *byte = region.bytes[addr - region.vaddr];
                return true;
            }
        }
        return false;
    }

    /* Get or declare a trace function for the given address. */
    llvm::Function *GetLiftedTraceDeclaration(uint64_t addr) override {
        auto it = traces_.find(addr);
        if (it != traces_.end()) {
            return it->second;
        }
        return nullptr;
    }

    /* Get or declare a lifted trace definition. */
    llvm::Function *GetLiftedTraceDefinition(uint64_t addr) override {
        return GetLiftedTraceDeclaration(addr);
    }

    /* Register a newly lifted trace. */
    bool TryAddTraceDeclaration(uint64_t addr, llvm::Function *func) {
        traces_[addr] = func;
        return true;
    }

    void SetLiftedTraceDefinition(uint64_t addr,
                                  llvm::Function *lifted_func) override {
        traces_[addr] = lifted_func;
    }

    void ForEachDevirtualizedTarget(
        const remill::Instruction &inst,
        std::function<void(uint64_t, remill::DevirtualizedTargetKind)> func)
        override {
        /* No devirtualization in Phase 0. */
        (void)inst;
        (void)func;
    }

private:
    const RemillContextImpl *ctx_;
    std::unordered_map<uint64_t, llvm::Function *> traces_;
};

extern "C" {

RemillContext *remill_context_create(const char *semantics_dir) {
    auto ctx = new RemillContextImpl();
    ctx->semantics_dir = semantics_dir ? semantics_dir : "";
    return reinterpret_cast<RemillContext *>(ctx);
}

void remill_context_destroy(RemillContext *ctx_opaque) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);
    delete ctx;
}

void remill_add_code_region(RemillContext *ctx_opaque,
                            const char *name,
                            uint64_t vaddr,
                            const uint8_t *bytes,
                            uint64_t size) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);
    CodeRegion region;
    region.name = name ? name : "";
    region.vaddr = vaddr;
    region.bytes.assign(bytes, bytes + size);
    ctx->code_regions.push_back(std::move(region));
}

void remill_add_data_section(RemillContext *ctx_opaque,
                             const char *name,
                             uint64_t vaddr,
                             const uint8_t *bytes,
                             uint64_t size,
                             int writable) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);
    DataSectionInfo section;
    section.name = name ? name : "";
    section.vaddr = vaddr;
    section.bytes.assign(bytes, bytes + size);
    section.writable = writable != 0;
    ctx->data_sections.push_back(std::move(section));
}

void remill_set_entry_point(RemillContext *ctx_opaque, uint64_t vaddr) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);
    ctx->entry_point = vaddr;
}

RemillModule *remill_lift(RemillContext *ctx_opaque) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);

    try {
        auto llvm_ctx = std::make_unique<llvm::LLVMContext>();

        /* Get the x86_64 Linux architecture. */
        auto arch = remill::Arch::Get(*llvm_ctx, "linux", "amd64");
        if (!arch) {
            ctx->last_error = "failed to get x86_64 Linux arch from Remill";
            return nullptr;
        }

        /* Load semantics module. */
        auto sem_module = remill::LoadArchSemantics(arch.get());
        if (!sem_module) {
            ctx->last_error = "failed to load Remill semantics from: " +
                              ctx->semantics_dir;
            return nullptr;
        }

        /* Create the trace manager. */
        Elf2WasmTraceManager manager(ctx);

        /* Create the trace lifter. */
        remill::TraceLifter lifter(arch.get(), &manager);

        /* Lift from the entry point. Remill will follow control flow. */
        if (!lifter.Lift(ctx->entry_point)) {
            ctx->last_error = "TraceLifter::Lift failed at entry 0x" +
                              std::to_string(ctx->entry_point);
            return nullptr;
        }

        /* Emit data sections as global constants/variables. */
        for (const auto &section : ctx->data_sections) {
            auto *data_type = llvm::ArrayType::get(
                llvm::Type::getInt8Ty(*llvm_ctx), section.bytes.size());
            auto *initializer = llvm::ConstantDataArray::get(
                *llvm_ctx, section.bytes);
            auto *gv = new llvm::GlobalVariable(
                *sem_module, data_type, !section.writable,
                llvm::GlobalValue::ExternalLinkage, initializer,
                "data_" + section.name);
            /* Place at the correct virtual address offset. */
            gv->setAlignment(llvm::Align(16));
        }

        /* Declare the syscall shim as an external function. */
        auto *i64_ty = llvm::Type::getInt64Ty(*llvm_ctx);
        std::vector<llvm::Type *> syscall_args(7, i64_ty);
        auto *syscall_ft = llvm::FunctionType::get(i64_ty, syscall_args, false);
        sem_module->getOrInsertFunction("__elf2wasm_syscall", syscall_ft);

        /* Verify the module. */
        std::string verify_err;
        llvm::raw_string_ostream verify_stream(verify_err);
        if (llvm::verifyModule(*sem_module, &verify_stream)) {
            ctx->last_error = "LLVM module verification failed: " + verify_err;
            return nullptr;
        }

        auto result = new RemillModuleImpl();
        result->llvm_ctx = std::move(llvm_ctx);
        result->module = std::move(sem_module);
        return reinterpret_cast<RemillModule *>(result);

    } catch (const std::exception &e) {
        ctx->last_error = std::string("exception during lifting: ") + e.what();
        return nullptr;
    }
}

void remill_module_destroy(RemillModule *module_opaque) {
    auto module = reinterpret_cast<RemillModuleImpl *>(module_opaque);
    delete module;
}

int remill_write_bc(RemillModule *module_opaque, const char *path) {
    auto module = reinterpret_cast<RemillModuleImpl *>(module_opaque);
    std::error_code ec;
    llvm::raw_fd_ostream os(path, ec, llvm::sys::fs::OF_None);
    if (ec) return 1;
    llvm::WriteBitcodeToFile(*module->module, os);
    os.flush();
    return os.has_error() ? 1 : 0;
}

int remill_write_ll(RemillModule *module_opaque, const char *path) {
    auto module = reinterpret_cast<RemillModuleImpl *>(module_opaque);
    std::error_code ec;
    llvm::raw_fd_ostream os(path, ec, llvm::sys::fs::OF_None);
    if (ec) return 1;
    module->module->print(os, nullptr);
    os.flush();
    return os.has_error() ? 1 : 0;
}

const char *remill_get_error(RemillContext *ctx_opaque) {
    auto ctx = reinterpret_cast<RemillContextImpl *>(ctx_opaque);
    if (ctx->last_error.empty()) return nullptr;
    return ctx->last_error.c_str();
}

} /* extern "C" */
