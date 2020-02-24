# elb-logs-to-cloudwatch

Initially this project sifts through ELB logs, extracts 502 and 503s and publish to cloudwatch with the
loadbalancer and target group as dimensions. It can be used to trigger alarms on cloudwatch in order to scale
services.

## Required configuration
This module requires that you have a running docker (to build the rust code for lambda) and
python (required by [terraform-aws-lambda](https://github.com/claranet/terraform-aws-lambda))

## Using with terraform
```hcl
module "elb_logs_to_cloud_watch" {
  source     = "github.com/devsbb/elb-logs-to-cloudwatch"
  aws_region = "eu-central-1"
  buckets    = ["bucket-a", "bucket-b"]
  pipelines = jsonencode([
    {
      filter = "elb_status_code in {502..503}",
      output = {
        type        = "cloudwatch_metric"
        metric_name = "BadGatewayRequestCount",
        namespace   = "Grover/LambdaParser"
      }
    },
    {
      filter = "elb_status_code > 0",
      output = {
        type = "stdout"
      }
    }
  ])
}
```
The final binary will be compiled and a zip will be uploaded to s3 in order to run the lambda.
In the future we will provide a pre-compiled binary to avoid depending on docker for the final deployment.
