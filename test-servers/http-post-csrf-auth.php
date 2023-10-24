<?php session_start(); ?>
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PHP CSRF</title>
</head>

<body>
    <main>
        <?php

        $request_method = strtoupper($_SERVER['REQUEST_METHOD']);

        if ($request_method === 'GET') {
            $_SESSION['token'] = bin2hex(random_bytes(35));
        ?>
            <form action="<?= htmlspecialchars($_SERVER['PHP_SELF']) ?>" method="post">
                <header>
                    <h1>Login</h1>
                </header>
                <div>
                    <label for="user">Username:</label>
                    <input type="text" name="user" value="" id="user">
                </div>

                <div>
                    <label for="pass">Password:</label>
                    <input type="password" name="pass" value="" id="pass">
                </div>

                <input type="hidden" name="token" value="<?= $_SESSION['token'] ?? '' ?>">
                <button type="submit">Login</button>
            </form>
        <?php
        } elseif ($request_method === 'POST') {
            $token = filter_input(INPUT_POST, 'token', FILTER_SANITIZE_STRING);
            if (!$token || $token !== $_SESSION['token']) {
                // show an error message
                echo '<p class="error">Error: invalid form submission</p>';
                http_response_code(405);
                die('Not Allowed');
            }

            $username = isset($_POST['user']) ? $_POST['user'] : '';
            $password = isset($_POST['pass']) ? $_POST['pass'] : '';

            if ($username != 'admin666' || $password != 'test12345') {
                echo '<p class="error">Error: invalid credentials</p>';
                http_response_code(403);
                die('Forbidden');
            }

            echo "OK";
        }
        ?>
    </main>
</body>

</html>