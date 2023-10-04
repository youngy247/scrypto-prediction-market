# RadBet - A Comprehensive Prediction Market Framework

This project delivers a blueprint focused on establishing a robust prediction market infrastructure. Investors and participants can utilize this platform to make predictions on various events and potentially earn returns based on the outcomes.

## Core Architecture

The core of the project is encapsulated within the primary blueprint, named the `PredictionMarketBlueprint`. It's designed with modularity and interaction in mind, to integrate seamlessly with various prediction models and market structures.

### Structure

- **PredictionMarket**: The main blueprint that sets up and manages the prediction market.

> **Note**: Further details on supporting components or additional blueprints will be added as the project expands.

## Building the Blueprints

1. Ensure your development environment is set up with the required tools, especially the `resim` tool. More information on setting this up can be found [here](https://docs.radixdlt.com/docs). For this project, you should be using Scrypto 1.0.0.

2. To build the project:
   - Navigate to the `prediction-market-blueprint` directory in your terminal.
   - Execute the command:
     ```bash
     cargo build
     ```

## Running the Test Suite

Ensure the `resim` tool is properly installed.

To run the tests:
- In the `prediction-market-blueprint` directory, run the following command:
  ```bash
  cargo test -- --test-threads=1

## Generating Documentation

For a comprehensive understanding of how the blueprints function, you can generate detailed documentation.

To generate the docs:

- In the **'prediction-market-blueprint'** directory, execute:
  ```bash
  cargo doc
  ```
- **Open in Browser**: If you run `cargo doc --open`, once the documentation is built, it will automatically open in your default web browser.


The generated web pages provide an extensive breakdown of the blueprint operations and functionalities.