load_results <- function(file) {
  data <- read.table(file, header=TRUE, sep=",")
  data$time <- data$time / 1000000 # ns -> ms
  data$file_size_human <- humanReadable(x=data$file_size, units="auto", standard="IEC", digits=0, width=NULL, sep=" ", justify="right")
  data$file_size_human <- factor(data$file_size_human, levels = unique(data$file_size_human[order(data$file_size)]))
  
  data
}

aggregate_results <- function(results, func) {
  data <- aggregate(results$time, list(results$file_size), func)
  colnames(data) <- c("file_size", "time")
  data
}

calc_speedups <- function(current, previous) {
  file_size <- previous$file_size
  speedup <- (previous$time - current$time) / previous$time * 100
  data.frame(file_size, speedup) %>%
    group_by(file_size) %>%
    summarise(
      n=n(),
      mean=mean(speedup),
      median=median(speedup),
      sd=sd(speedup)
    ) %>%
    mutate( se=sd / sqrt(n)) %>%
    mutate( ic=se * qt((1-alpha)/2 + .5, n-1))
}

hr <- function(x) {
  x <- replace(x, is.na(x), 0)
  humanReadable(x, units="auto", standard="IEC", digits=0, width=NULL, sep=" ", justify="right")
}

plot <- function(title, xlab, ylab) {
    ggplot() +
    theme_ipsum() +
    ggtitle(title) +
    xlab(xlab) +
    ylab(ylab)
}

conditional_coloring_plot <- function(title, xlab, ylab) {
  plot(title, xlab, ylab) +
    scale_fill_manual(guide = FALSE, breaks = c(TRUE, FALSE), values=c(current_color, previous_color)) +
    scale_color_manual(guide = FALSE, breaks = c(TRUE, FALSE), values=c(current_color, previous_color)) +
    theme(legend.position="none")
}

filter_outliers <- function(results, k) {
  s <- results %>%
    group_by(file_size) %>%
    summarise(
      p25=quantile(time)[2],
      p75=quantile(time)[4],
      iqr=IQR(time)
    )

  results %>% left_join(s, by = "file_size") %>% filter( !(time < p25 - (iqr * k) | time > p75 + (iqr * k)) ) 
}
