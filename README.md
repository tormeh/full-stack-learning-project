# full-stack-learning-project

Requires something a la this in postgres:
create database mydb;
create extension "uuid-ossp";
create table test (id uuid default uuid_generate_v4(), content text);
