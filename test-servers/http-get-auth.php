<?php
$username = isset($_GET['user']) ? $_GET['user'] : '';
$password = isset($_GET['pass']) ? $_GET['pass'] : '';

if ($username != 'admin666' || $password != 'test12345') {
    http_response_code(403);
    die('Forbidden');
}

echo "OK!";
