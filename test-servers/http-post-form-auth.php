<?php
$username = isset($_POST['user']) ? $_POST['user'] : '';
$password = isset($_POST['pass']) ? $_POST['pass'] : '';

if ($username != 'admin666' || $password != 'test12345') {
    http_response_code(403);
    die('Forbidden');
}

echo "OK!";
