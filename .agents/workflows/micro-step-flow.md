---
description: Micro-Step Flow (Reasoning & Implementation)
---

# Micro-Step Architecture & Implementation Flow

Use this workflow to implement the application one structure or function at a time. The goal is to maximize code quality, correctness, and user understanding by never writing a block of code without explicit prior discussion and approval of the design choices.

**1. Pitch the Structure/Function**
The AI (as Planner) presents the isolated piece of logic to build (e.g., "We need to design the `Card` initialization function"). 
- Present the expected public interface (input types, output types, errors).
- Propose the internal mechanism (e.g., "We will use ULID for sorting").
- State the "Why" behind the choices.

**2. Debate & Approve**
The human driver reviews the proposal, asks questions, requests changes, and ultimately approves the architectural choice.

**3. Implement & Localize**
The AI (as Implementer) writes ONLY the exact piece of code approved in step 2. No sprawling file generation. No implementing neighboring functions.

**4. Unit Test (Zero-Trust)**
The AI (as Tester) generates the isolated `#[cfg(test)]` block to immediately verify the function works as designed, especially testing its failure modes. 

**5. Review & Document**
The human driver reviews the written code and passing tests. 
The AI MUST proactively add documentation following a strict 3-tier structure:
- **Module-Level (`//!`)**: Ensure a high-level `//!` block exists at the top of the file explaining the purpose of the module and giving references to the `docs/rust-for-python-devs.md` guide where appropriate.
- **Structural (`///`)**: Every public struct/method gets a `///` docstring with a `# Examples` block.
- **No Clutter**: Avoid conversational `//` inline comments. Keep the internal logic clean.

**6. Orchestrator Summary**
Once the implementation is finalized, the Orchestrator MUST provide a concise, high-level summary of exactly *what* was built and *why* those architectural decisions were made, so the human can easily log or review the step's impact. Once satisfied, the human specifies the next single function or block to work on, starting the loop over.
