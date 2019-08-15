load_results <- function(file) {
  data <- read.table(file, header=TRUE, sep=",")
  data$time <- data$time / 1000000 # ns -> ms
  data$file_size_human <- humanReadable(x=data$file_size, units="auto", standard="IEC", digits=0, width=NULL, sep=" ", justify="right")
  data$file_size_human <- factor(data$file_size_human, levels = unique(data$file_size_human[order(data$file_size)]))
  
  data
}

mean_results <- function(results) {
  means <- aggregate(results$time, list(results$file_size), mean)
  colnames(means) <- c("file_size", "mean_time")
  means
}

hr <- function(x) {
  x <- replace(x, is.na(x), 0)
  humanReadable(x, units="auto", standard="IEC", digits=0, width=NULL, sep=" ", justify="right")
}

