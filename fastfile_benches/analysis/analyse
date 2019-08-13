#!/usr/bin/env Rscript

library("optparse")
library("rmarkdown")

option_list = list(
    make_option(c("-f", "--file"), type="character", default=NULL, help="dataset file name", metavar="character"),
    make_option(c("-o", "--out"), type="character", default="out.txt", help="output file name [default= %default]", metavar="character")
); 

opt_parser = OptionParser(option_list=option_list);
opt = parse_args(opt_parser);

if (is.null(opt$file)){
  print_help(opt_parser)
  stop("At least one argument must be supplied (input file).n", call=FALSE)
}

render("fastfile_read_analysis.rmd",
       output_file = "../results/current/current",
       clean = TRUE,
       params = list(
                     data_file = "../results/current/current.csv"
                )
)

