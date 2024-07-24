<?php
if (isset($_GET['input'])) {
    $input = $_GET['input'];
    echo "Input: " . $input; // No sanitization
} else {
    echo 'No input provided.';
}
?>
