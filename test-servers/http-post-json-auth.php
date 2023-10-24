<?php
$payload = json_decode(file_get_contents('php://input'), true);
$username = isset($payload['user']) ? $payload['user'] : '';
$password = isset($payload['pass']) ? $payload['pass'] : '';

if ($username != 'admin666' || $password != 'test12345') {
    http_response_code(403);
    die('Forbidden');
}

echo "OK!";
