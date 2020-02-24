module "lambda" {
  source = "github.com/claranet/terraform-aws-lambda"

  function_name                  = "elb-logs-to-cloudwatch"
  description                    = "Process ELB logs from S3 to sift through bad gateways, timeouts and service unavailable"
  handler                        = "main.lambda_handler"
  runtime                        = "provided"
  timeout                        = 300
  reserved_concurrent_executions = 1

  // Specify a file or directory for the source code.
  source_path   = "${abspath(path.module)}/src"
  build_command = "${abspath(path.module)}/build-lambda.sh '$filename' '$runtime' '$source'"

  // Attach a policy.
  policy = {
    json = data.aws_iam_policy_document.lambda.json
  }

  // Add environment variables.
  environment = {
    variables = {
      // TODO: Remove dependency from this as the event has the bucket
      BUCKET_NAME    = ""
      RUST_BACKTRACE = 1
      INSIDE_LAMBDA  = 1
      PIPELINES      = var.pipelines
    }
  }
}
