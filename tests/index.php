<?php
header('HTTP/1.1 200 OK');
var_dump($_SERVER);
var_dump(file_get_contents('php://input'));
echo time(), PHP_EOL;
