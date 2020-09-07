## Deps

    pip3 install pandas tensorflow seaborn matplotlib

## Generate Training Data

From project root:

    # collect best paths sampled across all initial states in a grid
    cargo run -p generate-data --bin generate-data --release

    # process paths to create a simple csv with label (the cost/time), and
    # initial player state
    cargo run -p generate-data --bin time ./data/generated time.csv

## Train

    cd heuristic/train/nn
    python3 train-nn.py ../../../time.csv
