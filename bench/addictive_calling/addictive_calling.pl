sub increment { $_[0] + 1 }

my $i = 0;
while ($i < 10000000) {
    $i = increment($i);
}
