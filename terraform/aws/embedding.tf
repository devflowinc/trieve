
resource "aws_instance" "new-deployment-embeddigs" {
  # Deep Learning Base OSS Nvidia Driver GPU AMI (Ubuntu 22.04) 20240501
  # https://aws.amazon.com/releasenotes/aws-deep-learning-base-gpu-ami-ubuntu-22-04/
  ami           = "ami-0b5b200dc06507fcb"
  
  instance_type = "g4dn.xlarge"
  user_data     = templatefile("./embedding_server_cloud_init.yaml", { ssh_key : file(var.ssh_pub_key_file) }) # Cloudinit

  subnet_id                   = module.vpc.public_subnets[0]
  vpc_security_group_ids      = [aws_security_group.embeddings.id]
  associate_public_ip_address = true

  root_block_device {
    volume_size = 200 # In GB
    volume_type = "gp3"
  }

  tags = {
    Name = "${var.cluster-name}-embeddings"
  }
}

resource "aws_security_group" "embeddings" {
  name   = "embeddings"
  vpc_id = module.vpc.vpc_id

  # SSH access from the VPC
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
   
  ingress {
    from_port   = 5000
    to_port     = 5000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 6000
    to_port     = 6000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 7000
    to_port     = 7000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 8000
    to_port     = 8000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 9000
    to_port     = 9000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
