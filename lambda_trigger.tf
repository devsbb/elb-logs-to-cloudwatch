resource "aws_lambda_permission" "allow_bucket" {
  count         = length(var.buckets)
  statement_id  = "AllowExecutionFromS3Bucket"
  action        = "lambda:InvokeFunction"
  function_name = module.lambda.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = data.aws_s3_bucket.elb[count.index].arn
}

data "aws_s3_bucket" "elb" {
  count  = length(var.buckets)
  bucket = var.buckets[count.index]
}

resource "aws_s3_bucket_notification" "bucket_notification" {
  count  = length(var.buckets)
  bucket = var.buckets[count.index]

  lambda_function {
    lambda_function_arn = module.lambda.function_arn
    events              = ["s3:ObjectCreated:*"]
  }
}
