data "aws_iam_policy_document" "lambda" {
  statement {
    actions = [
      "cloudwatch:PutMetricData",
      "logs:PutLogEvents",
      "logs:CreateLogStream",
    ]

    resources = ["*"]
  }

  statement {
    actions   = ["s3:GetObject"]
    resources = formatlist("%s/*", data.aws_s3_bucket.elb.*.arn)
  }

  statement {
    actions   = ["s3:ListBucket"]
    resources = data.aws_s3_bucket.elb.*.arn
  }
}
