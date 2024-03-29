# Introduction

This is a simple portfolio chooser based on time series of investment funds, based
on the [Sharpe Ratio](https://en.wikipedia.org/wiki/Sharpe_ratio) and the
[Efficient Frontier](https://en.wikipedia.org/wiki/Efficient_frontier).

This is part of a dual study trying to compare the implementation of the same data
science project in Python and Rust. Check out the [Python repository](https://github.com/AloizioMacedo/investments)
in case you are interested.

## Configuration

Main parameters for the run of the portfolio chooser can be selected in the
`config/config.toml` file.

## Running

You can directly run the full pipeline with

```bash
make run
```

or directly

```bash
cargo run -r
```

and check the visualization of the efficient frontier with

```bash
make viz
```

or

```bash
make viz_hull
```

In order to run the visualization commands, you need Firefox. In case you don't have
it, just open the corresponding htmls in the data folder directly instead.

## Pipeline

Each part of the pipeline can be run separately through the according bin.

### Raw files

For the fund time series, we are currently capturing monthly data by using
<https://dados.cvm.gov.br/dataset/fii-doc-inf_mensal>. This seems to be restricted
only to real estate, so we probably want to expand this in the future.

Meanwhile, since we aren't able to download this data programatically or structurally,
we copy the tables from XP funds list for each year separately to an excel spreadsheet
and export it as csv. In order for the pipeline to work, we establish that these files
should be names as `{CNPJ}_{YEAR}.csv`, where `{CNPJ}` switches the slash char `/` for
an underline. As an example, this would be a valid name: `32.319.351_0001-56_2023.csv`

For the CDI time series, we capture the data directly by copy-pasting the data
in the following link: <https://brasilindicadores.com.br/cdi/>.

### Preprocessed files

Preprocessing transforms the rentability into a simple multiplier, e.g. a monthly
rentability of `+1.2%` gets translated into `1.012` on a `"values"` column.

A `"dt"` column contains date in the format `YYYY-MM-01`. The funds-related csv,
`"funds.csv"` also has an additional column `"CNPJ_Fundo"`, corresponding to an
identifier of the fund (c.f. <https://www.gov.br/receitafederal/pt-br/servicos/cadastro/cnpj>).

To run this part of the pipeline, run

```bash
cargo run -r --bin preprocess
```

### Timeseries

Consists of JSONs of the time series of each fund.

To run this part of the pipeline, run

```bash
cargo run -r --bin timeseries
```

### Outputs

We get the risk-return plots in `.html` format.

Furthermore, we get the convex hull of the plot to more easily identify the efficient
frontier.

We also get the optimal allocation (with respect to the Sharpe ratio) in JSON format.

To run this part of the pipeline, run

```bash
cargo run -r --bin outputs
```
