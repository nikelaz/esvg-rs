# ESVG

ESVG is a powerful SVG optimization tool designed to reduce file sizes without compromising the visual quality of rendered graphics. It leverages Competitive Transformer Optimization^[1]^ to explore multiple optimization strategies and select the most efficient one.

## Competitive Transformer Optimization ^[1]^

At the heart of ESVG's optimization process is the Arbiter, which acts as the central orchestrator. The Arbiter passes the input SVG through a series of transformers, each a specialized plugin that applies various techniques to improve the SVG. These transformers can make a wide range of changes, such as simplifying paths, removing unnecessary elements, or applying different encoding schemes.

The key innovation of ESVG is its use of competitive optimization. Instead of applying a single transformation, the Arbiter evaluates multiple transformers simultaneously. After each transformer processes the SVG, the Arbiter compares the results and keeps the transformation that achieves the most efficient output. Efficiency, in this case, is defined by the reduction in file size while ensuring that the rendered vector graphics appear identical or imperceptibly different.

The process works as follows:

- **Input**: The Arbiter receives an SVG file as input.
- **Transformation**: The input SVG is passed through multiple transformers, which apply various optimizations.
- **Comparison**: The Arbiter compares the output from each transformer, assessing which result has the smallest file size while maintaining visual integrity.
- **Selection**: The Arbiter selects the most efficient output and proceeds with it.

This approach ensures that only the most effective transformations are applied, producing the smallest possible file size with no perceivable visible quality loss.

## State

**This project is in very early development. Most of the features are not implemented and it cannot be used effectively for any optimization.**

