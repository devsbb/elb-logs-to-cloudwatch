variable "aws_region" {}
variable "buckets" { type = list(string) }
variable "pipelines" { type = string }
variable "reserved_concurrent_executions" { default = 1 }
