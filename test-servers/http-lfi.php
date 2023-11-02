<?php
$page = isset($_GET['page']) ? $_GET['page'] : '';

if (file_exists($page))
    require_once $page;
