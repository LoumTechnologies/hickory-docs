terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  required_version = ">= 1.5"
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region to deploy to"
  type        = string
  default     = "us-east-1"
}

variable "relay_image" {
  description = "Docker image URI for hick-relay (ECR or Docker Hub)"
  type        = string
}

variable "relay_domain" {
  description = "Domain name for the relay (e.g. relay.hickory.dev)"
  type        = string
  default     = "relay.hickory.dev"
}

variable "certificate_arn" {
  description = "ACM certificate ARN for TLS on the relay domain"
  type        = string
}

# ---------------------------------------------------------------------------
# Networking
# ---------------------------------------------------------------------------

data "aws_vpc" "default" {
  default = true
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

# ---------------------------------------------------------------------------
# ECS Cluster
# ---------------------------------------------------------------------------

resource "aws_ecs_cluster" "relay" {
  name = "hick-relay"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# ---------------------------------------------------------------------------
# IAM
# ---------------------------------------------------------------------------

resource "aws_iam_role" "relay_task" {
  name = "hick-relay-task"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "relay_task_exec" {
  role       = aws_iam_role.relay_task.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

# ---------------------------------------------------------------------------
# CloudWatch log group
# ---------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "relay" {
  name              = "/ecs/hick-relay"
  retention_in_days = 30
}

# ---------------------------------------------------------------------------
# ECS Task Definition
# ---------------------------------------------------------------------------

resource "aws_ecs_task_definition" "relay" {
  family                   = "hick-relay"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.relay_task.arn
  task_role_arn            = aws_iam_role.relay_task.arn

  container_definitions = jsonencode([{
    name      = "hick-relay"
    image     = var.relay_image
    essential = true
    portMappings = [{ containerPort = 8080, protocol = "tcp" }]
    environment = [{ name = "RUST_LOG", value = "info" }]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.relay.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "relay"
      }
    }
  }])
}

# ---------------------------------------------------------------------------
# Security groups
# ---------------------------------------------------------------------------

resource "aws_security_group" "alb" {
  name   = "hick-relay-alb"
  vpc_id = data.aws_vpc.default.id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "relay" {
  name   = "hick-relay-ecs"
  vpc_id = data.aws_vpc.default.id

  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# ---------------------------------------------------------------------------
# ALB (with WebSocket support — no idle timeout limitation)
# ---------------------------------------------------------------------------

resource "aws_lb" "relay" {
  name               = "hick-relay"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = data.aws_subnets.default.ids

  # WebSocket connections can be long-lived; set a generous idle timeout.
  idle_timeout = 3600
}

resource "aws_lb_target_group" "relay" {
  name        = "hick-relay"
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = data.aws_vpc.default.id
  target_type = "ip"

  health_check {
    path                = "/new"
    healthy_threshold   = 2
    unhealthy_threshold = 3
    interval            = 30
  }
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.relay.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.relay.arn
  }
}

# ---------------------------------------------------------------------------
# ECS Service
# ---------------------------------------------------------------------------

resource "aws_ecs_service" "relay" {
  name            = "hick-relay"
  cluster         = aws_ecs_cluster.relay.id
  task_definition = aws_ecs_task_definition.relay.arn
  desired_count   = 1
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = data.aws_subnets.default.ids
    security_groups  = [aws_security_group.relay.id]
    assign_public_ip = true
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.relay.arn
    container_name   = "hick-relay"
    container_port   = 8080
  }

  depends_on = [aws_lb_listener.https]
}

# ---------------------------------------------------------------------------
# Outputs
# ---------------------------------------------------------------------------

output "relay_dns" {
  description = "ALB DNS name — point relay.hickory.dev CNAME here"
  value       = aws_lb.relay.dns_name
}

output "relay_url" {
  description = "Public HTTPS URL for the relay"
  value       = "https://${var.relay_domain}"
}
